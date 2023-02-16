use crate::{Disk, Error, Machine, Result};
use fslock::LockFile;
use path_macro::path;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Values, HashMap},
    env,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::Command,
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
const DISK_DIR_PATH: &str = "disks";
const DISK_LOCK_DIR_PATH: &str = "disk-locks";
const MACHINE_DIR_PATH: &str = "machines";
const MACHINE_LOCK_DIR_PATH: &str = "machine-locks";
const DISK_BACKUP_DIR_PATH: &str = "disk-backups";
const MACHINE_BACKUP_DIR_PATH: &str = "machine-backups";

#[derive(Deserialize, Serialize)]
pub struct Library {
    #[serde(skip)]
    path: PathBuf,
    disks: HashMap<String, Disk>,
    disk_backups: HashMap<String, Disk>,
    machines: HashMap<String, Machine>,
    machine_backups: HashMap<String, Machine>,
}

impl Library {
    fn uefi_path(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(env::var(UEFI_ENV_VAR)?))
    }

    fn state_path(&self) -> Result<PathBuf> {
        Ok(path![self.path / STATE_PATH])
    }

    fn disk_dir_path(&self) -> Result<PathBuf> {
        Ok(path![self.path / DISK_DIR_PATH])
    }

    fn disk_path(&self, disk: &Disk) -> Result<PathBuf> {
        Ok(path![
            self.disk_dir_path()? / format!("{}.qcow2", disk.name)
        ])
    }

    fn disk_lock_dir_path(&self) -> Result<PathBuf> {
        Ok(path![self.path / DISK_LOCK_DIR_PATH])
    }

    fn disk_lock_path(&self, disk: &Disk) -> Result<PathBuf> {
        Ok(path![
            self.disk_lock_dir_path()? / format!("{}.lock", disk.name)
        ])
    }

    fn disk_backup_dir_path(&self) -> Result<PathBuf> {
        Ok(path![self.path / DISK_BACKUP_DIR_PATH])
    }

    fn disk_backup_path(&self, disk: &Disk) -> Result<PathBuf> {
        Ok(path![
            self.disk_backup_dir_path()? / format!("{}.qcow2", disk.name)
        ])
    }

    fn machine_dir_path(&self) -> Result<PathBuf> {
        Ok(path![self.path / MACHINE_DIR_PATH])
    }

    fn machine_path(&self, machine: &Machine) -> Result<PathBuf> {
        Ok(path![
            self.machine_dir_path()? / format!("{}.qcow2", machine.name)
        ])
    }

    fn machine_lock_dir_path(&self) -> Result<PathBuf> {
        Ok(path![self.path / MACHINE_LOCK_DIR_PATH])
    }

    fn machine_lock_path(&self, machine: &Machine) -> Result<PathBuf> {
        Ok(path![
            self.machine_lock_dir_path()? / format!("{}.lock", machine.name)
        ])
    }

    fn machine_backup_dir_path(&self) -> Result<PathBuf> {
        Ok(path![self.path / MACHINE_BACKUP_DIR_PATH])
    }

    fn machine_backup_path(&self, machine: &Machine) -> Result<PathBuf> {
        Ok(path![
            self.machine_backup_dir_path()? / format!("{}.qcow2", machine.name)
        ])
    }

    fn setup(&self) -> Result<()> {
        fs::create_dir_all(&self.path)?;
        fs::create_dir_all(self.disk_dir_path()?)?;
        fs::create_dir_all(self.disk_lock_dir_path()?)?;
        fs::create_dir_all(self.disk_backup_dir_path()?)?;
        fs::create_dir_all(self.machine_dir_path()?)?;
        fs::create_dir_all(self.machine_lock_dir_path()?)?;
        fs::create_dir_all(self.machine_backup_dir_path()?)?;
        Ok(())
    }

    fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let state = Self {
            path: path.into(),
            disks: HashMap::new(),
            disk_backups: HashMap::new(),
            machines: HashMap::new(),
            machine_backups: HashMap::new(),
        };
        state.setup()?;
        Ok(state)
    }

    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Into<PathBuf> + Clone,
    {
        let mut state = Library::new(path.clone())?;
        if fs::metadata(&state.state_path()?).is_ok() {
            state = toml::from_str(&fs::read_to_string(state.state_path()?)?)?;
            state.path = path.into();
        }
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        fs::write(self.state_path()?, toml::to_string(self)?)?;
        Ok(())
    }

    fn allocate_qcow2<P>(&self, name: P, size: usize) -> Result<()>
    where
        P: AsRef<Path> + Display,
    {
        let mut cmd = Command::new(QEMU_IMAGER);
        cmd.arg("create");
        cmd.args(["-f", "qcow2"]);
        cmd.arg(&format!("{name}"));
        cmd.arg(&format!("{size}G"));

        let mut child = cmd.spawn()?;
        child.wait()?;

        Ok(())
    }

    pub fn add_disk(&mut self, disk: Disk) -> Result<()> {
        self.disks.insert(disk.name.clone(), disk.clone());

        self.allocate_qcow2(
            self.disk_path(&disk)?.to_str().ok_or(Error::BadPath)?,
            disk.size,
        )
    }

    fn base_qemu_command(&self, machine: &Machine, cores: usize, ram: usize) -> Result<Command> {
        let mut cmd = Command::new(QEMU_RUNNER);
        cmd.args(["-M", "virt,highmem=on"]);
        cmd.args(["-accel", "hvf"]);
        cmd.args(["-cpu", "host"]);
        cmd.args(["-smp", &format!("{}", cores)]);
        cmd.args(["-m", &format!("{}G", ram)]);
        cmd.args(["-bios", self.uefi_path()?.to_str().ok_or(Error::BadPath)?]);
        cmd.args([
            "-drive",
            &format!(
                "file={},if=none,cache=writethrough,id=hd0",
                self.machine_path(&machine)?
                    .to_str()
                    .ok_or(Error::BadPath)?
            ),
        ]);
        cmd.args(["-device", "virtio-gpu-pci"]);
        cmd.args(["-device", "virtio-blk-device,drive=hd0"]);
        cmd.args(["-net", &format!("user,hostfwd=tcp::{}-:22", machine.port)]);
        cmd.args(["-net", "nic"]);
        cmd.arg("-nographic");
        Ok(cmd)
    }

    pub fn add_machine(&mut self, machine: Machine) -> Result<()> {
        self.machines.insert(machine.name.clone(), machine.clone());

        self.allocate_qcow2(
            self.machine_path(&machine)?
                .to_str()
                .ok_or(Error::BadPath)?,
            machine.size,
        )
    }

    pub fn get_disk(&self, name: &str) -> Result<&Disk> {
        self.disks
            .get(name)
            .ok_or(Error::InvalidDisk { name: name.into() })
    }

    fn lock_disk(&self, disk: &Disk) -> Result<LockFile> {
        let mut lock = LockFile::open(&self.disk_lock_path(disk)?)?;
        if lock.try_lock()? {
            Ok(lock)
        } else {
            Err(Error::InUse)
        }
    }

    pub fn remove_disk(&mut self, name: &str) -> Result<()> {
        let disk = self.get_disk(name)?.clone();
        let mut lock = self.lock_disk(&disk)?;
        self.disks.remove(&disk.name);
        fs::remove_file(self.disk_lock_path(&disk)?)?;
        lock.unlock()?;
        Ok(())
    }

    pub fn get_machine(&self, name: &str) -> Result<&Machine> {
        self.machines
            .get(name)
            .ok_or(Error::InvalidMachine { name: name.into() })
    }

    fn lock_machine(&self, machine: &Machine) -> Result<LockFile> {
        let mut lock = LockFile::open(&self.machine_lock_path(machine)?)?;
        if lock.try_lock()? {
            Ok(lock)
        } else {
            Err(Error::InUse)
        }
    }

    pub fn remove_machine(&mut self, name: &str) -> Result<()> {
        let machine = self.get_machine(name)?.clone();
        let mut lock = self.lock_machine(&machine)?;
        self.machines.remove(&machine.name);
        fs::remove_file(self.machine_lock_path(&machine)?)?;
        lock.unlock()?;
        Ok(())
    }

    pub fn run_machine(
        &mut self,
        name: String,
        cores: usize,
        ram: usize,
        disks: &[String],
        iso: Option<PathBuf>,
    ) -> Result<()> {
        let machine = self.get_machine(&name)?;

        let mut machine_lock = self.lock_machine(machine)?;
        let mut disk_locks = Vec::new();

        let mut cmd = self.base_qemu_command(machine, cores, ram)?;

        for disk in disks {
            let disk = self
                .disks
                .get(disk)
                .ok_or(Error::InvalidDisk { name: disk.clone() })?;
            let disk_path = self.disk_path(&disk)?;

            cmd.args([
                "-drive",
                &format!(
                    "file={},format=qcow2,media=disk",
                    disk_path.to_str().ok_or(Error::BadPath)?
                ),
            ]);

            let disk_lock = self.lock_disk(disk)?;
            disk_locks.push(disk_lock);
        }

        if let Some(iso) = iso {
            cmd.args(["-cdrom", iso.to_str().ok_or(Error::BadPath)?]);
        }

        let mut child = cmd.spawn()?;
        child.wait()?;

        machine_lock.unlock()?;

        for mut disk_lock in disk_locks.drain(..) {
            disk_lock.unlock()?;
        }

        Ok(())
    }

    pub fn get_machine_port(&self, name: String) -> Result<u16> {
        self.get_machine(&name).map(|machine| machine.port)
    }

    pub fn disks(&self) -> Values<String, Disk> {
        self.disks.values()
    }

    pub fn machines(&self) -> Values<String, Machine> {
        self.machines.values()
    }

    pub fn disk_backups(&self) -> Values<String, Disk> {
        self.disk_backups.values()
    }

    pub fn machine_backups(&self) -> Values<String, Machine> {
        self.machine_backups.values()
    }

    pub fn backup_disk(&mut self, name: String) -> Result<()> {
        let disk = self.get_disk(&name)?;
        let disk_path = self.disk_path(&disk)?;
        let backup_path = self.disk_backup_path(&disk)?;
        fs::copy(disk_path, backup_path)?;
        self.disk_backups.insert(name, disk.clone());
        Ok(())
    }

    pub fn backup_machine(&mut self, name: String) -> Result<()> {
        let machine = self.get_machine(&name)?;
        let machine_path = self.machine_path(&machine)?;
        let backup_path = self.machine_backup_path(&machine)?;
        fs::copy(machine_path, backup_path)?;
        self.machine_backups.insert(name, machine.clone());
        Ok(())
    }

    pub fn ports(&self) -> impl Iterator<Item = u16> + '_ {
        self.machines.values().map(|machine| machine.port)
    }
}
