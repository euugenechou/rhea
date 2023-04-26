use crate::{disk::Disk, error::Error, machine::Machine, result::Result, snapshot::Snapshot};
use fslock::LockFile;
use path_macro::path;
use piper::PipedCommand;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map::Values, BTreeMap},
    env,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str,
};

#[cfg(target_arch = "x86-64")]
const QEMU_RUNNER: &str = "qemu-system-x86_64";
#[cfg(target_arch = "mips")]
const QEMU_RUNNER: &str = "qemu-system-mips";
#[cfg(target_arch = "powerpc")]
const QEMU_RUNNER: &str = "qemu-system-ppc";
#[cfg(target_arch = "powerpc64")]
const QEMU_RUNNER: &str = "qemu-system-ppc64";
#[cfg(target_arch = "arm")]
const QEMU_RUNNER: &str = "qemu-system-arm";
#[cfg(target_arch = "aarch64")]
const QEMU_RUNNER: &str = "qemu-system-aarch64";
const QEMU_IMAGER: &str = "qemu-img";
const UEFI_ENV_VAR: &str = "RHEA_UEFI_PATH";
const STATE_PATH: &str = "state.toml";
const PROCESS_LOCK_PATH: &str = ".proc.lock";
const DISK_DIR_PATH: &str = "disks";
const MACHINE_DIR_PATH: &str = "machines";
const SNAPSHOT_DIR_PATH: &str = "snapshots";

#[derive(Deserialize, Serialize)]
pub struct State {
    #[serde(skip)]
    path: PathBuf,
    disks: BTreeMap<String, Disk>,
    machines: BTreeMap<String, Machine>,
    snapshots: BTreeMap<String, Snapshot>,
}

impl State {
    fn uefi_path(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(env::var(UEFI_ENV_VAR)?))
    }

    fn state_path(&self) -> PathBuf {
        path![self.path / STATE_PATH]
    }

    fn process_lock_path(&self) -> PathBuf {
        path![self.path / PROCESS_LOCK_PATH]
    }

    fn disk_dir_path(&self) -> PathBuf {
        path![self.path / DISK_DIR_PATH]
    }

    fn disk_path(&self, name: &str) -> PathBuf {
        path![self.disk_dir_path() / format!("{}.qcow2", name)]
    }

    fn machine_dir_path(&self) -> PathBuf {
        path![self.path / MACHINE_DIR_PATH]
    }

    fn machine_path(&self, name: &str) -> PathBuf {
        path![self.machine_dir_path() / format!("{}.qcow2", name)]
    }

    fn snapshot_dir_path(&self) -> PathBuf {
        path![self.path / SNAPSHOT_DIR_PATH]
    }

    fn snapshot_path(&self, name: &str) -> PathBuf {
        path![self.snapshot_dir_path() / format!("{}.qcow2", name)]
    }

    fn setup(&self) -> Result<()> {
        fs::create_dir_all(&self.path)?;
        fs::create_dir_all(self.disk_dir_path())?;
        fs::create_dir_all(self.machine_dir_path())?;
        fs::create_dir_all(self.snapshot_dir_path())?;
        Ok(())
    }

    fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let state = Self {
            path: path.into(),
            disks: BTreeMap::new(),
            machines: BTreeMap::new(),
            snapshots: BTreeMap::new(),
        };
        state.setup()?;
        Ok(state)
    }

    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Into<PathBuf> + Clone,
    {
        let mut state = State::new(path.clone())?;
        if fs::metadata(&state.state_path()).is_ok() {
            state = toml::from_str(&fs::read_to_string(state.state_path())?)?;
            state.path = path.into();
        }
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        fs::write(self.state_path(), toml::to_string(self)?)?;
        Ok(())
    }

    fn allocate_qcow2<P>(&self, name: P, size: usize) -> Result<()>
    where
        P: AsRef<Path> + Display,
    {
        Command::new(QEMU_IMAGER)
            .arg("create")
            .args(["-f", "qcow2"])
            .arg(&format!("{name}"))
            .arg(&format!("{size}G"))
            .spawn()?
            .wait()?;
        Ok(())
    }

    fn base_qemu_command<P: AsRef<Path>>(
        &self,
        resource: P,
        port: u16,
        cores: usize,
        ram: usize,
    ) -> Result<Command> {
        let mut cmd = Command::new(QEMU_RUNNER);
        cmd.args(["-M", "virt,highmem=on"])
            .args(["-accel", "hvf"])
            .args(["-cpu", "host"])
            .args(["-smp", &format!("{}", cores)])
            .args(["-m", &format!("{}G", ram)])
            .args([
                "-bios",
                self.uefi_path()?.to_str().ok_or(Error::InvalidPath {
                    path: self.uefi_path()?,
                })?,
            ])
            .args([
                "-drive",
                &format!(
                    "file={},if=none,cache=writethrough,id=hd0",
                    resource.as_ref().to_str().ok_or(Error::InvalidPath {
                        path: resource.as_ref().into()
                    })?
                ),
            ])
            .args(["-device", "virtio-gpu-pci"])
            .args(["-device", "virtio-blk-device,drive=hd0"])
            .args(["-net", &format!("user,hostfwd=tcp::{port}-:22")])
            .args(["-net", "nic"])
            .arg("-nographic");
        Ok(cmd)
    }

    fn get_process_lock(&self) -> Result<LockFile> {
        let mut lock = LockFile::open(&self.process_lock_path())?;
        lock.lock()?;
        Ok(lock)
    }

    fn resource_in_use<P: AsRef<Path>>(&self, resource: P) -> Result<bool> {
        let mut lock = self.get_process_lock()?;

        let in_use = PipedCommand::run(format!(
            "ps aux | grep -v grep | grep {}",
            resource.as_ref().to_str().ok_or(Error::InvalidPath {
                path: resource.as_ref().into()
            })?
        ))?
        .status
        .code()
            == Some(0);

        lock.unlock()?;

        Ok(in_use)
    }

    pub fn disk_in_use(&self, name: &str) -> Result<bool> {
        if !self.disks.contains_key(name) {
            return Err(Error::InvalidDisk { name: name.into() });
        }
        self.resource_in_use(self.disk_path(name))
    }

    pub fn machine_in_use(&self, name: &str) -> Result<bool> {
        if !self.machines.contains_key(name) {
            return Err(Error::InvalidMachine { name: name.into() });
        }
        self.resource_in_use(self.machine_path(name))
    }

    pub fn snapshot_in_use(&self, name: &str) -> Result<bool> {
        if !self.snapshots.contains_key(name) {
            return Err(Error::InvalidSnapshot { name: name.into() });
        }
        self.resource_in_use(self.snapshot_path(name))
    }

    pub fn add_disk(&mut self, name: &str, size: usize) -> Result<()> {
        if self.disks.contains_key(name) {
            return Err(Error::DiskExists { name: name.into() });
        }

        self.disks.insert(
            name.into(),
            Disk {
                name: name.into(),
                size,
            },
        );

        self.allocate_qcow2(
            self.disk_path(name).to_str().ok_or(Error::InvalidPath {
                path: self.disk_path(name),
            })?,
            size,
        )
    }

    pub fn get_disk(&self, name: &str) -> Result<&Disk> {
        self.disks
            .get(name)
            .ok_or(Error::InvalidDisk { name: name.into() })
    }

    pub fn remove_disk(&mut self, name: &str) -> Result<()> {
        if self.disk_in_use(name)? {
            return Err(Error::DiskInUse { name: name.into() });
        }
        self.disks.remove(name);
        Ok(())
    }

    pub fn add_machine(&mut self, name: &str, port: u16, size: usize) -> Result<()> {
        if self.machines.contains_key(name) {
            return Err(Error::MachineExists { name: name.into() });
        }

        self.machines.insert(
            name.into(),
            Machine {
                name: name.into(),
                port,
                size,
            },
        );

        self.allocate_qcow2(
            self.machine_path(name).to_str().ok_or(Error::InvalidPath {
                path: self.machine_path(name),
            })?,
            size,
        )
    }

    pub fn get_machine(&self, name: &str) -> Result<&Machine> {
        self.machines
            .get(name)
            .ok_or(Error::InvalidMachine { name: name.into() })
    }

    pub fn remove_machine(&mut self, name: &str) -> Result<()> {
        if self.machine_in_use(name)? {
            return Err(Error::MachineInUse { name: name.into() });
        }
        self.machines.remove(name);
        Ok(())
    }

    pub fn add_snapshot(&mut self, name: &str, base: &str) -> Result<()> {
        if self.snapshots.contains_key(name) {
            return Err(Error::SnapshotExists { name: name.into() });
        }

        let machine = self.get_machine(base)?;

        self.snapshots.insert(
            name.into(),
            Snapshot {
                name: name.into(),
                base: base.into(),
                port: machine.port,
                size: machine.size,
            },
        );

        Command::new(QEMU_IMAGER)
            .arg("create")
            .args(["-f", "qcow2"])
            .args([
                "-b",
                &format!(
                    "{}",
                    self.machine_path(base).to_str().ok_or(Error::InvalidPath {
                        path: self.machine_path(base)
                    })?
                ),
            ])
            .args(["-F", "qcow2"])
            .arg(&format!(
                "{}",
                self.snapshot_path(name)
                    .to_str()
                    .ok_or(Error::InvalidPath {
                        path: self.snapshot_path(name)
                    })?
            ))
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn get_snapshot(&self, name: &str) -> Result<&Snapshot> {
        self.snapshots
            .get(name)
            .ok_or(Error::InvalidSnapshot { name: name.into() })
    }

    pub fn remove_snapshot(&mut self, name: &str) -> Result<()> {
        if self.snapshot_in_use(name)? {
            return Err(Error::SnapshotInUse { name: name.into() });
        }
        self.snapshots.remove(name);
        Ok(())
    }

    pub fn disks(&self) -> Values<String, Disk> {
        self.disks.values()
    }

    pub fn machines(&self) -> Values<String, Machine> {
        self.machines.values()
    }

    pub fn snapshots(&self) -> Values<String, Snapshot> {
        self.snapshots.values()
    }

    pub fn start(
        &mut self,
        name: &str,
        cores: usize,
        ram: usize,
        foreground: bool,
        snapshot: bool,
        disks: &[String],
        iso: Option<PathBuf>,
    ) -> Result<()> {
        let (resource, port) = if snapshot {
            let snapshot = self.get_snapshot(name)?;
            (self.snapshot_path(&snapshot.name), snapshot.port)
        } else {
            let machine = self.get_machine(name)?;
            (self.machine_path(&machine.name), machine.port)
        };

        let mut cmd = self.base_qemu_command(&resource, port, cores, ram)?;

        for disk in disks {
            if self.disk_in_use(disk)? {
                return Err(Error::DiskInUse { name: disk.into() });
            }

            cmd.args([
                "-drive",
                &format!(
                    "file={},format=qcow2,media=disk",
                    self.disk_path(disk).to_str().ok_or(Error::InvalidPath {
                        path: self.disk_path(disk)
                    })?
                ),
            ]);
        }

        if let Some(iso) = iso {
            cmd.args([
                "-cdrom",
                iso.to_str()
                    .ok_or(Error::InvalidPath { path: iso.clone() })?,
            ]);
        }

        if !foreground {
            cmd.stdout(Stdio::null());
        }

        let mut child = cmd.spawn()?;

        if foreground {
            child.wait()?;
        }

        Ok(())
    }

    pub fn stop(&self, name: &str, snapshot: bool) -> Result<()> {
        if snapshot && !self.snapshot_in_use(name)? {
            return Err(Error::SnapshotNotInUse { name: name.into() });
        } else if !self.machine_in_use(name)? {
            return Err(Error::MachineNotInUse { name: name.into() });
        }

        let resource = if snapshot {
            self.snapshot_path(name)
        } else {
            self.machine_path(name)
        };

        let output = PipedCommand::run(format!(
            "ps aux | grep -v grep | grep {}",
            resource.to_str().ok_or(Error::InvalidPath {
                path: resource.clone()
            })?
        ))?;

        let pid = str::from_utf8(&output.stdout)
            .map_err(|_| Error::MachineNotInUse { name: name.into() })?
            .lines()
            .next()
            .ok_or(Error::MachineNotInUse { name: name.into() })?
            .split_whitespace()
            .skip(1)
            .next()
            .unwrap();

        Command::new("kill").arg(&pid).spawn()?.wait()?;

        Ok(())
    }

    pub fn connect(
        &self,
        name: &str,
        username: Option<String>,
        forward_keys: bool,
        snapshot: bool,
    ) -> Result<()> {
        let port = if snapshot {
            self.get_snapshot(name)?.port
        } else {
            self.get_machine(name)?.port
        };

        let mut cmd = Command::new("ssh");

        if forward_keys {
            cmd.arg("-A");
        }

        cmd.arg(&format!("-p {}", port))
            .arg(&format!(
                "{}@localhost",
                if let Some(username) = username {
                    username
                } else {
                    env::var("USER")?
                }
            ))
            .spawn()?
            .wait()?;

        Ok(())
    }
}
