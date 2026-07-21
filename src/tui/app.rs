use std::path::PathBuf;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::core::backup::{BackupConfig, BackupEngine, BackupHeader, BackupPhase, BackupProgress};
use crate::core::cloner::{CloneConfig, ClonePhase, CloneProgress, Cloner};
use crate::core::flasher::{FlashConfig, FlashPhase, FlashProgress, Flasher};
use crate::core::partition::{
    EraseMethod, EraseProgress, FsType, PartitionInfo, PartitionManager, TableType,
};
use crate::device::detect::DeviceInfo;
use crate::tui::ui::file_browser::FileBrowser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `Screen`
pub enum Screen {
    /// Variante d'énumération `Home` du type énuméré.
    Home,
    /// Variante d'énumération `Flash` du type énuméré.
    Flash,
    /// Variante d'énumération `Clone` du type énuméré.
    Clone,
    /// Variante d'énumération `Backup` du type énuméré.
    Backup,
    /// Variante d'énumération `Restore` du type énuméré.
    Restore,
    /// Variante d'énumération `Partition` du type énuméré.
    Partition,
    /// Variante d'énumération `Settings` du type énuméré.
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `FlashState`
pub enum FlashState {
    /// Variante d'énumération `SelectImage` du type énuméré.
    SelectImage,
    /// Variante d'énumération `SelectTarget` du type énuméré.
    SelectTarget,
    /// Variante d'énumération `Confirm` du type énuméré.
    Confirm,
    /// Variante d'énumération `Writing` du type énuméré.
    Writing,
    /// Variante d'énumération `Verifying` du type énuméré.
    Verifying,
    /// Variante d'énumération `Done` du type énuméré.
    Done,
    /// Variante d'énumération `Failed` du type énuméré.
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `CloneState`
pub enum CloneState {
    /// Variante d'énumération `SelectSource` du type énuméré.
    SelectSource,
    /// Variante d'énumération `SelectDest` du type énuméré.
    SelectDest,
    /// Variante d'énumération `Confirm` du type énuméré.
    Confirm,
    /// Variante d'énumération `Copying` du type énuméré.
    Copying,
    /// Variante d'énumération `Verifying` du type énuméré.
    Verifying,
    /// Variante d'énumération `Done` du type énuméré.
    Done,
    /// Variante d'énumération `Failed` du type énuméré.
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `BackupState`
pub enum BackupState {
    /// Variante d'énumération `SelectSource` du type énuméré.
    SelectSource,
    /// Variante d'énumération `SelectOutput` du type énuméré.
    SelectOutput,
    /// Variante d'énumération `Confirm` du type énuméré.
    Confirm,
    /// Variante d'énumération `Running` du type énuméré.
    Running,
    /// Variante d'énumération `Done` du type énuméré.
    Done,
    /// Variante d'énumération `Failed` du type énuméré.
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `RestoreState`
pub enum RestoreState {
    /// Variante d'énumération `SelectInput` du type énuméré.
    SelectInput,
    /// Variante d'énumération `ShowHeader` du type énuméré.
    ShowHeader,
    /// Variante d'énumération `SelectTarget` du type énuméré.
    SelectTarget,
    /// Variante d'énumération `Confirm` du type énuméré.
    Confirm,
    /// Variante d'énumération `Running` du type énuméré.
    Running,
    /// Variante d'énumération `Done` du type énuméré.
    Done,
    /// Variante d'énumération `Failed` du type énuméré.
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `PartitionState`
pub enum PartitionState {
    /// Variante d'énumération `SelectDevice` du type énuméré.
    SelectDevice,
    /// Variante d'énumération `ShowTable` du type énuméré.
    ShowTable,
    /// Variante d'énumération `SelectAction` du type énuméré.
    SelectAction,
    /// Variante d'énumération `InputParams` du type énuméré.
    InputParams,
    /// Variante d'énumération `Confirm` du type énuméré.
    Confirm,
    /// Variante d'énumération `Running` du type énuméré.
    Running,
    /// Variante d'énumération `Done` du type énuméré.
    Done,
    /// Variante d'énumération `Failed` du type énuméré.
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Énumération publique `PartitionActionType`
pub enum PartitionActionType {
    /// Variante d'énumération `Add` du type énuméré.
    Add,
    /// Variante d'énumération `Delete` du type énuméré.
    Delete,
    /// Variante d'énumération `Format` du type énuméré.
    Format,
    /// Variante d'énumération `CreateTable` du type énuméré.
    CreateTable,
    /// Variante d'énumération `Erase` du type énuméré.
    Erase,
}

/// Structure publique `OperationProgress`
pub struct OperationProgress {
    /// Champ public `bytes_written` de la structure correspondante.
    pub bytes_written: u64,
    /// Champ public `total_bytes` de la structure correspondante.
    pub total_bytes: u64,
    /// Champ public `speed_bytes_per_sec` de la structure correspondante.
    pub speed_bytes_per_sec: f64,
    /// Champ public `eta_seconds` de la structure correspondante.
    pub eta_seconds: f64,
}

/// Structure publique `App`
pub struct App {
    /// Champ public `screen` de la structure correspondante.
    pub screen: Screen,
    /// Champ public `devices` de la structure correspondante.
    pub devices: Vec<DeviceInfo>,
    /// Champ public `selected_index` de la structure correspondante.
    pub selected_index: usize,
    /// Champ public `status_message` de la structure correspondante.
    pub status_message: Option<String>,
    /// Champ public `file_browser` de la structure correspondante.
    pub file_browser: Option<FileBrowser>,

    /// État global du flux de flash actif.
    pub flash_state: FlashState,
    /// Champ public `progress` de la structure correspondante.
    pub progress: Option<OperationProgress>,
    /// Champ public `selected_image` de la structure correspondante.
    pub selected_image: Option<PathBuf>,
    /// Champ public `selected_target` de la structure correspondante.
    pub selected_target: Option<String>,
    /// Champ public `flash_error` de la structure correspondante.
    pub flash_error: Option<String>,
    progress_rx: Option<mpsc::Receiver<FlashProgress>>,
    flash_start_time: Option<Instant>,

    /// État global du flux de clone actif.
    pub clone_state: CloneState,
    /// Champ public `clone_source` de la structure correspondante.
    pub clone_source: Option<String>,
    /// Champ public `clone_dest` de la structure correspondante.
    pub clone_dest: Option<PathBuf>,
    /// Champ public `clone_error` de la structure correspondante.
    pub clone_error: Option<String>,
    /// Champ public `clone_progress` de la structure correspondante.
    pub clone_progress: Option<OperationProgress>,
    clone_progress_rx: Option<mpsc::Receiver<CloneProgress>>,

    /// État global du flux de backup actif.
    pub backup_state: BackupState,
    /// Champ public `backup_source` de la structure correspondante.
    pub backup_source: Option<String>,
    /// Champ public `backup_output` de la structure correspondante.
    pub backup_output: Option<PathBuf>,
    /// Champ public `backup_error` de la structure correspondante.
    pub backup_error: Option<String>,
    /// Champ public `backup_progress` de la structure correspondante.
    pub backup_progress: Option<OperationProgress>,
    backup_progress_rx: Option<mpsc::Receiver<BackupProgress>>,

    /// État global du flux de restauration actif.
    pub restore_state: RestoreState,
    /// Champ public `restore_input` de la structure correspondante.
    pub restore_input: Option<PathBuf>,
    /// Champ public `restore_target` de la structure correspondante.
    pub restore_target: Option<String>,
    /// Champ public `restore_header` de la structure correspondante.
    pub restore_header: Option<BackupHeader>,
    /// Champ public `restore_error` de la structure correspondante.
    pub restore_error: Option<String>,
    /// Champ public `restore_progress` de la structure correspondante.
    pub restore_progress: Option<OperationProgress>,
    restore_progress_rx: Option<mpsc::Receiver<BackupProgress>>,

    /// État global du flux de partition actif.
    pub partition_state: PartitionState,
    /// Champ public `partition_device` de la structure correspondante.
    pub partition_device: Option<String>,
    /// Champ public `partition_table` de la structure correspondante.
    pub partition_table: Vec<PartitionInfo>,
    /// Champ public `partition_table_type` de la structure correspondante.
    pub partition_table_type: Option<TableType>,
    /// Champ public `partition_action` de la structure correspondante.
    pub partition_action: Option<PartitionActionType>,
    /// Champ public `partition_error` de la structure correspondante.
    pub partition_error: Option<String>,
    /// Champ public `partition_selected` de la structure correspondante.
    pub partition_selected: usize,
    /// Champ public `partition_input` de la structure correspondante.
    pub partition_input: String,
    /// Champ public `partition_input_field` de la structure correspondante.
    pub partition_input_field: u8, // 0=type, 1=size, 2=label
    /// Champ public `partition_progress` de la structure correspondante.
    pub partition_progress: Option<OperationProgress>,
    partition_erase_rx: Option<mpsc::Receiver<EraseProgress>>,

    // Speed tracking (shared)
    last_bytes: u64,
    last_speed_update: Option<Instant>,
    current_speed: f64,
}

impl App {
    /// Fonction publique `new`
    pub fn new() -> Self {
        Self {
            screen: Screen::Home,
            devices: Vec::new(),
            selected_index: 0,
            status_message: None,
            file_browser: None,

            flash_state: FlashState::SelectImage,
            progress: None,
            selected_image: None,
            selected_target: None,
            flash_error: None,
            progress_rx: None,
            flash_start_time: None,

            clone_state: CloneState::SelectSource,
            clone_source: None,
            clone_dest: None,
            clone_error: None,
            clone_progress: None,
            clone_progress_rx: None,

            backup_state: BackupState::SelectSource,
            backup_source: None,
            backup_output: None,
            backup_error: None,
            backup_progress: None,
            backup_progress_rx: None,

            restore_state: RestoreState::SelectInput,
            restore_input: None,
            restore_target: None,
            restore_header: None,
            restore_error: None,
            restore_progress: None,
            restore_progress_rx: None,

            partition_state: PartitionState::SelectDevice,
            partition_device: None,
            partition_table: Vec::new(),
            partition_table_type: None,
            partition_action: None,
            partition_error: None,
            partition_selected: 0,
            partition_input: String::new(),
            partition_input_field: 0,
            partition_progress: None,
            partition_erase_rx: None,

            last_bytes: 0,
            last_speed_update: None,
            current_speed: 0.0,
        }
    }

    fn reset_speed_tracking(&mut self) {
        self.last_bytes = 0;
        self.last_speed_update = None;
        self.current_speed = 0.0;
    }

    fn compute_speed(&mut self, bytes: u64) -> (f64, f64, u64) {
        let now = Instant::now();
        if let Some(last_time) = self.last_speed_update {
            let dt = now.duration_since(last_time).as_secs_f64();
            if dt > 0.2 {
                let delta = bytes.saturating_sub(self.last_bytes);
                let instant_speed = delta as f64 / dt;
                self.current_speed = self.current_speed * 0.7 + instant_speed * 0.3;
                self.last_bytes = bytes;
                self.last_speed_update = Some(now);
            }
        } else {
            self.last_speed_update = Some(now);
            self.last_bytes = bytes;
        }
        (self.current_speed, 0.0, bytes)
    }
}

impl App {
    /// Fonction publique `tick`
    pub fn tick(&mut self) {
        // Flash
        let mut flash_updates = Vec::new();
        if let Some(ref mut rx) = self.progress_rx {
            while let Ok(p) = rx.try_recv() {
                flash_updates.push(p);
            }
        }
        for p in flash_updates {
            self.update_flash_progress(p);
        }

        // Clone
        let mut clone_updates = Vec::new();
        if let Some(ref mut rx) = self.clone_progress_rx {
            while let Ok(p) = rx.try_recv() {
                clone_updates.push(p);
            }
        }
        for p in clone_updates {
            self.update_clone_progress(p);
        }

        // Backup
        let mut backup_updates = Vec::new();
        if let Some(ref mut rx) = self.backup_progress_rx {
            while let Ok(p) = rx.try_recv() {
                backup_updates.push(p);
            }
        }
        for p in backup_updates {
            self.update_backup_progress(p);
        }

        // Restore
        let mut restore_updates = Vec::new();
        if let Some(ref mut rx) = self.restore_progress_rx {
            while let Ok(p) = rx.try_recv() {
                restore_updates.push(p);
            }
        }
        for p in restore_updates {
            self.update_restore_progress(p);
        }

        // Partition erase
        let mut erase_updates = Vec::new();
        if let Some(ref mut rx) = self.partition_erase_rx {
            while let Ok(p) = rx.try_recv() {
                erase_updates.push(p);
            }
        }
        for p in erase_updates {
            self.update_erase_progress(p);
        }
    }

    fn update_flash_progress(&mut self, p: FlashProgress) {
        let (speed, _, _) = self.compute_speed(p.bytes_written);
        let eta = if speed > 0.0 && p.total_bytes > p.bytes_written {
            (p.total_bytes - p.bytes_written) as f64 / speed
        } else {
            0.0
        };

        match p.phase {
            FlashPhase::Writing => {
                self.flash_state = FlashState::Writing;
                self.progress = Some(OperationProgress {
                    bytes_written: p.bytes_written,
                    total_bytes: p.total_bytes,
                    speed_bytes_per_sec: speed,
                    eta_seconds: eta,
                });
            }
            FlashPhase::Verifying => {
                self.flash_state = FlashState::Verifying;
                self.progress = Some(OperationProgress {
                    bytes_written: 0,
                    total_bytes: p.total_bytes,
                    speed_bytes_per_sec: 0.0,
                    eta_seconds: 0.0,
                });
            }
            FlashPhase::Done => {
                self.flash_state = FlashState::Done;
                self.progress = None;
                self.progress_rx = None;
                self.status_message = Some("Flash complete! Press Esc to return.".into());
            }
            FlashPhase::Failed => {
                self.flash_state = FlashState::Failed;
                self.progress = None;
                self.progress_rx = None;
                self.status_message = Some("Flash failed! Press Esc to return.".into());
            }
            FlashPhase::Preparing => {
                self.flash_state = FlashState::Writing;
                self.status_message = Some("Preparing...".into());
            }
        }
    }

    fn update_clone_progress(&mut self, p: CloneProgress) {
        let (speed, _, _) = self.compute_speed(p.bytes_copied);
        let eta = if speed > 0.0 && p.total_bytes > p.bytes_copied {
            (p.total_bytes - p.bytes_copied) as f64 / speed
        } else {
            0.0
        };

        match p.phase {
            ClonePhase::Analyzing => {
                self.clone_state = CloneState::Copying;
                self.status_message = Some("Analyzing...".into());
            }
            ClonePhase::Copying => {
                self.clone_state = CloneState::Copying;
                self.clone_progress = Some(OperationProgress {
                    bytes_written: p.bytes_copied,
                    total_bytes: p.total_bytes,
                    speed_bytes_per_sec: speed,
                    eta_seconds: eta,
                });
            }
            ClonePhase::Verifying => {
                self.clone_state = CloneState::Verifying;
                self.clone_progress = Some(OperationProgress {
                    bytes_written: 0,
                    total_bytes: p.total_bytes,
                    speed_bytes_per_sec: 0.0,
                    eta_seconds: 0.0,
                });
            }
            ClonePhase::Done => {
                self.clone_state = CloneState::Done;
                self.clone_progress = None;
                self.clone_progress_rx = None;
                self.status_message = Some("Clone complete!".into());
            }
            ClonePhase::Failed => {
                self.clone_state = CloneState::Failed;
                self.clone_progress = None;
                self.clone_progress_rx = None;
                self.clone_error = Some("Clone failed".into());
            }
        }
    }

    fn update_backup_progress(&mut self, p: BackupProgress) {
        let (speed, _, _) = self.compute_speed(p.bytes_processed);
        let eta = if speed > 0.0 && p.total_bytes > p.bytes_processed {
            (p.total_bytes - p.bytes_processed) as f64 / speed
        } else {
            0.0
        };

        match p.phase {
            BackupPhase::Analyzing => {
                self.status_message = Some("Analyzing...".into());
            }
            BackupPhase::Reading | BackupPhase::Compressing => {
                self.backup_state = BackupState::Running;
                self.backup_progress = Some(OperationProgress {
                    bytes_written: p.bytes_processed,
                    total_bytes: p.total_bytes,
                    speed_bytes_per_sec: speed,
                    eta_seconds: eta,
                });
            }
            BackupPhase::Done => {
                self.backup_state = BackupState::Done;
                self.backup_progress = None;
                self.backup_progress_rx = None;
                self.status_message = Some("Backup complete!".into());
            }
            BackupPhase::Failed => {
                self.backup_state = BackupState::Failed;
                self.backup_progress = None;
                self.backup_progress_rx = None;
                self.backup_error = Some("Backup failed".into());
            }
        }
    }

    fn update_restore_progress(&mut self, p: BackupProgress) {
        let (speed, _, _) = self.compute_speed(p.bytes_processed);
        let eta = if speed > 0.0 && p.total_bytes > p.bytes_processed {
            (p.total_bytes - p.bytes_processed) as f64 / speed
        } else {
            0.0
        };

        match p.phase {
            BackupPhase::Analyzing => {
                self.status_message = Some("Analyzing backup...".into());
            }
            BackupPhase::Reading | BackupPhase::Compressing => {
                self.restore_state = RestoreState::Running;
                self.restore_progress = Some(OperationProgress {
                    bytes_written: p.bytes_processed,
                    total_bytes: p.total_bytes,
                    speed_bytes_per_sec: speed,
                    eta_seconds: eta,
                });
            }
            BackupPhase::Done => {
                self.restore_state = RestoreState::Done;
                self.restore_progress = None;
                self.restore_progress_rx = None;
                self.status_message = Some("Restore complete!".into());
            }
            BackupPhase::Failed => {
                self.restore_state = RestoreState::Failed;
                self.restore_progress = None;
                self.restore_progress_rx = None;
                self.restore_error = Some("Restore failed".into());
            }
        }
    }

    fn update_erase_progress(&mut self, p: EraseProgress) {
        let (speed, _, _) = self.compute_speed(p.bytes_erased);
        let eta = if speed > 0.0 && p.total_bytes > p.bytes_erased {
            (p.total_bytes - p.bytes_erased) as f64 / speed
        } else {
            0.0
        };

        self.partition_progress = Some(OperationProgress {
            bytes_written: p.bytes_erased,
            total_bytes: p.total_bytes,
            speed_bytes_per_sec: speed,
            eta_seconds: eta,
        });
        self.status_message = Some(format!("Pass {}/{}", p.pass, p.total_passes));

        if p.bytes_erased >= p.total_bytes && p.pass >= p.total_passes {
            self.partition_state = PartitionState::Done;
            self.partition_progress = None;
            self.partition_erase_rx = None;
            self.status_message = Some("Secure erase complete!".into());
        }
    }

    // ── Start operations ──────────────────────────────────────────

    /// Fonction publique `start_flash`
    pub fn start_flash(&mut self) {
        let image = match self.selected_image {
            Some(ref p) => p.clone(),
            None => return,
        };
        let tgt = match self.selected_target {
            Some(ref t) => t.clone(),
            None => return,
        };

        let (tx, rx) = mpsc::channel(128);
        self.progress_rx = Some(rx);
        self.flash_start_time = Some(Instant::now());
        self.flash_state = FlashState::Writing;
        self.reset_speed_tracking();

        tokio::spawn(async move {
            let config = FlashConfig {
                block_size: 4 * 1024 * 1024,
                verify: true,
                auto_unmount: true,
            };
            let flasher = Flasher::new(config);
            let _ = crate::device::mount::ensure_unmounted(&tgt).await;

            if let Err(e) = flasher.flash(&image, &tgt, tx.clone()).await {
                tracing::error!(error = %e, "Flash failed");
                let _ = tx
                    .send(FlashProgress {
                        device_index: 0,
                        device_name: tgt,
                        bytes_written: 0,
                        total_bytes: 0,
                        phase: FlashPhase::Failed,
                    })
                    .await;
            }
        });
    }

    /// Fonction publique `start_clone`
    pub fn start_clone(&mut self) {
        let src = match self.clone_source {
            Some(ref s) => s.clone(),
            None => return,
        };
        let dst = match self.clone_dest {
            Some(ref d) => d.to_string_lossy().to_string(),
            None => return,
        };

        let (tx, rx) = mpsc::channel(128);
        self.clone_progress_rx = Some(rx);
        self.clone_state = CloneState::Copying;
        self.reset_speed_tracking();

        tokio::spawn(async move {
            let config = CloneConfig::default();
            let cloner = Cloner::new(config);
            if let Err(e) = cloner.clone_device(&src, &dst, tx.clone()).await {
                tracing::error!(error = %e, "Clone failed");
                let _ = tx
                    .send(CloneProgress {
                        bytes_copied: 0,
                        total_bytes: 0,
                        phase: ClonePhase::Failed,
                    })
                    .await;
            }
        });
    }

    /// Fonction publique `start_backup`
    pub fn start_backup(&mut self) {
        let src = match self.backup_source {
            Some(ref s) => s.clone(),
            None => return,
        };
        let out = match self.backup_output {
            Some(ref o) => o.to_string_lossy().to_string(),
            None => return,
        };

        let (tx, rx) = mpsc::channel(128);
        self.backup_progress_rx = Some(rx);
        self.backup_state = BackupState::Running;
        self.reset_speed_tracking();

        tokio::spawn(async move {
            let config = BackupConfig::default();
            let engine = BackupEngine::new(config);
            if let Err(e) = engine.create_backup(&src, &out, tx.clone()).await {
                tracing::error!(error = %e, "Backup failed");
                let _ = tx
                    .send(BackupProgress {
                        bytes_processed: 0,
                        total_bytes: 0,
                        phase: BackupPhase::Failed,
                    })
                    .await;
            }
        });
    }

    /// Fonction publique `start_restore`
    pub fn start_restore(&mut self) {
        let inp = match self.restore_input {
            Some(ref i) => i.to_string_lossy().to_string(),
            None => return,
        };
        let tgt = match self.restore_target {
            Some(ref t) => t.clone(),
            None => return,
        };

        let (tx, rx) = mpsc::channel(128);
        self.restore_progress_rx = Some(rx);
        self.restore_state = RestoreState::Running;
        self.reset_speed_tracking();

        tokio::spawn(async move {
            let config = BackupConfig::default();
            let engine = BackupEngine::new(config);
            let _ = crate::device::mount::ensure_unmounted(&tgt).await;
            if let Err(e) = engine.restore_backup(&inp, &tgt, tx.clone()).await {
                tracing::error!(error = %e, "Restore failed");
                let _ = tx
                    .send(BackupProgress {
                        bytes_processed: 0,
                        total_bytes: 0,
                        phase: BackupPhase::Failed,
                    })
                    .await;
            }
        });
    }

    // ── Key handling ──────────────────────────────────────────────

    /// Fonction publique `handle_key`
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.screen {
            Screen::Home => self.handle_home_key(key),
            Screen::Flash => self.handle_flash_key(key),
            Screen::Clone => self.handle_clone_key(key),
            Screen::Backup => self.handle_backup_key(key),
            Screen::Restore => self.handle_restore_key(key),
            Screen::Partition => self.handle_partition_key(key),
            _ => {
                if key.code == KeyCode::Esc {
                    self.screen = Screen::Home;
                }
                false
            }
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return true,
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.screen = Screen::Flash;
                self.flash_state = FlashState::SelectImage;
                self.selected_image = None;
                self.selected_target = None;
                self.flash_error = None;
                self.progress = None;
                self.selected_index = 0;
                self.init_image_browser();
                self.refresh_devices();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.screen = Screen::Clone;
                self.clone_state = CloneState::SelectSource;
                self.clone_source = None;
                self.clone_dest = None;
                self.clone_error = None;
                self.clone_progress = None;
                self.selected_index = 0;
                self.refresh_devices();
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.screen = Screen::Backup;
                self.backup_state = BackupState::SelectSource;
                self.backup_source = None;
                self.backup_output = None;
                self.backup_error = None;
                self.backup_progress = None;
                self.selected_index = 0;
                self.refresh_devices();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.screen = Screen::Restore;
                self.restore_state = RestoreState::SelectInput;
                self.restore_input = None;
                self.restore_target = None;
                self.restore_header = None;
                self.restore_error = None;
                self.restore_progress = None;
                self.selected_index = 0;
                self.init_rfb_browser();
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.screen = Screen::Partition;
                self.partition_state = PartitionState::SelectDevice;
                self.partition_device = None;
                self.partition_table.clear();
                self.partition_table_type = None;
                self.partition_action = None;
                self.partition_error = None;
                self.partition_selected = 0;
                self.partition_input.clear();
                self.partition_input_field = 0;
                self.partition_progress = None;
                self.selected_index = 0;
                self.refresh_devices();
            }
            KeyCode::Char('s') | KeyCode::Char('S') => self.screen = Screen::Settings,
            _ => {}
        }
        false
    }

    fn handle_flash_key(&mut self, key: KeyEvent) -> bool {
        match self.flash_state {
            FlashState::SelectImage => match key.code {
                KeyCode::Esc => {
                    self.screen = Screen::Home;
                    self.flash_state = FlashState::SelectImage;
                }
                KeyCode::Up => self.browser_up(),
                KeyCode::Down => self.browser_down(),
                KeyCode::Enter => {
                    if let Some(ref mut fb) = self.file_browser {
                        if let Some(path) = fb.selected_path().map(|p| p.to_owned()) {
                            self.selected_image = Some(path);
                            self.flash_state = FlashState::SelectTarget;
                            self.selected_index = 0;
                        } else {
                            fb.enter_selected();
                        }
                    }
                }
                _ => {}
            },
            FlashState::SelectTarget => match key.code {
                KeyCode::Esc => self.flash_state = FlashState::SelectImage,
                KeyCode::Up => self.selected_index = self.selected_index.saturating_sub(1),
                KeyCode::Down => {
                    let max = self.devices.len().saturating_sub(1);
                    self.selected_index = self.selected_index.saturating_add(1).min(max);
                }
                KeyCode::Enter => {
                    if let Some(dev) = self.devices.get(self.selected_index) {
                        self.selected_target = Some(dev.path.clone());
                        self.flash_state = FlashState::Confirm;
                    }
                }
                _ => {}
            },
            FlashState::Confirm => match key.code {
                KeyCode::Esc => self.flash_state = FlashState::SelectTarget,
                KeyCode::Enter => self.start_flash(),
                _ => {}
            },
            FlashState::Writing | FlashState::Verifying => {}
            FlashState::Done | FlashState::Failed => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.screen = Screen::Home;
                    self.flash_state = FlashState::SelectImage;
                    self.progress = None;
                    self.status_message = None;
                }
                _ => {}
            },
        }
        false
    }

    fn handle_clone_key(&mut self, key: KeyEvent) -> bool {
        match self.clone_state {
            CloneState::SelectSource => match key.code {
                KeyCode::Esc => self.screen = Screen::Home,
                KeyCode::Up => self.selected_index = self.selected_index.saturating_sub(1),
                KeyCode::Down => {
                    let max = self.devices.len().saturating_sub(1);
                    self.selected_index = self.selected_index.saturating_add(1).min(max);
                }
                KeyCode::Enter => {
                    if let Some(dev) = self.devices.get(self.selected_index) {
                        self.clone_source = Some(dev.path.clone());
                        self.clone_state = CloneState::SelectDest;
                        self.selected_index = 0;
                        self.init_all_browser();
                    }
                }
                _ => {}
            },
            CloneState::SelectDest => match key.code {
                KeyCode::Esc => {
                    self.clone_state = CloneState::SelectSource;
                    self.selected_index = 0;
                }
                KeyCode::Up => self.browser_up(),
                KeyCode::Down => self.browser_down(),
                KeyCode::Enter => {
                    if let Some(ref mut fb) = self.file_browser {
                        if let Some(path) = fb.selected_path().map(|p| p.to_owned()) {
                            self.clone_dest = Some(path);
                            self.clone_state = CloneState::Confirm;
                        } else {
                            fb.enter_selected();
                        }
                    }
                }
                _ => {}
            },
            CloneState::Confirm => match key.code {
                KeyCode::Esc => self.clone_state = CloneState::SelectDest,
                KeyCode::Enter => self.start_clone(),
                _ => {}
            },
            CloneState::Copying | CloneState::Verifying => {}
            CloneState::Done | CloneState::Failed => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.screen = Screen::Home;
                    self.clone_state = CloneState::SelectSource;
                    self.clone_progress = None;
                    self.status_message = None;
                }
                _ => {}
            },
        }
        false
    }

    fn handle_backup_key(&mut self, key: KeyEvent) -> bool {
        match self.backup_state {
            BackupState::SelectSource => match key.code {
                KeyCode::Esc => self.screen = Screen::Home,
                KeyCode::Up => self.selected_index = self.selected_index.saturating_sub(1),
                KeyCode::Down => {
                    let max = self.devices.len().saturating_sub(1);
                    self.selected_index = self.selected_index.saturating_add(1).min(max);
                }
                KeyCode::Enter => {
                    if let Some(dev) = self.devices.get(self.selected_index) {
                        self.backup_source = Some(dev.path.clone());
                        self.backup_state = BackupState::SelectOutput;
                        self.init_all_browser();
                    }
                }
                _ => {}
            },
            BackupState::SelectOutput => match key.code {
                KeyCode::Esc => {
                    self.backup_state = BackupState::SelectSource;
                    self.selected_index = 0;
                }
                KeyCode::Up => self.browser_up(),
                KeyCode::Down => self.browser_down(),
                KeyCode::Enter => {
                    // Select current directory as output location
                    if let Some(ref fb) = self.file_browser {
                        let source_name = self
                            .backup_source
                            .as_deref()
                            .unwrap_or("backup")
                            .rsplit('/')
                            .next()
                            .unwrap_or("backup");
                        let output = fb.current_dir.join(format!("{source_name}.rfb"));
                        self.backup_output = Some(output);
                        self.backup_state = BackupState::Confirm;
                    }
                }
                _ => {}
            },
            BackupState::Confirm => match key.code {
                KeyCode::Esc => self.backup_state = BackupState::SelectOutput,
                KeyCode::Enter => self.start_backup(),
                _ => {}
            },
            BackupState::Running => {}
            BackupState::Done | BackupState::Failed => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.screen = Screen::Home;
                    self.backup_state = BackupState::SelectSource;
                    self.backup_progress = None;
                    self.status_message = None;
                }
                _ => {}
            },
        }
        false
    }

    fn handle_restore_key(&mut self, key: KeyEvent) -> bool {
        match self.restore_state {
            RestoreState::SelectInput => match key.code {
                KeyCode::Esc => self.screen = Screen::Home,
                KeyCode::Up => self.browser_up(),
                KeyCode::Down => self.browser_down(),
                KeyCode::Enter => {
                    if let Some(ref mut fb) = self.file_browser {
                        if let Some(path) = fb.selected_path().map(|p| p.to_owned()) {
                            // Try to read header
                            match BackupEngine::read_header(path.to_str().unwrap_or("")) {
                                Ok(header) => {
                                    self.restore_input = Some(path);
                                    self.restore_header = Some(header);
                                    self.restore_state = RestoreState::ShowHeader;
                                }
                                Err(e) => {
                                    self.status_message = Some(format!("Invalid backup: {e}"));
                                }
                            }
                        } else {
                            fb.enter_selected();
                        }
                    }
                }
                _ => {}
            },
            RestoreState::ShowHeader => match key.code {
                KeyCode::Esc => self.restore_state = RestoreState::SelectInput,
                KeyCode::Enter => {
                    self.restore_state = RestoreState::SelectTarget;
                    self.selected_index = 0;
                    self.refresh_devices();
                }
                _ => {}
            },
            RestoreState::SelectTarget => match key.code {
                KeyCode::Esc => self.restore_state = RestoreState::ShowHeader,
                KeyCode::Up => self.selected_index = self.selected_index.saturating_sub(1),
                KeyCode::Down => {
                    let max = self.devices.len().saturating_sub(1);
                    self.selected_index = self.selected_index.saturating_add(1).min(max);
                }
                KeyCode::Enter => {
                    if let Some(dev) = self.devices.get(self.selected_index) {
                        self.restore_target = Some(dev.path.clone());
                        self.restore_state = RestoreState::Confirm;
                    }
                }
                _ => {}
            },
            RestoreState::Confirm => match key.code {
                KeyCode::Esc => self.restore_state = RestoreState::SelectTarget,
                KeyCode::Enter => self.start_restore(),
                _ => {}
            },
            RestoreState::Running => {}
            RestoreState::Done | RestoreState::Failed => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.screen = Screen::Home;
                    self.restore_state = RestoreState::SelectInput;
                    self.restore_progress = None;
                    self.status_message = None;
                }
                _ => {}
            },
        }
        false
    }

    fn handle_partition_key(&mut self, key: KeyEvent) -> bool {
        match self.partition_state {
            PartitionState::SelectDevice => match key.code {
                KeyCode::Esc => self.screen = Screen::Home,
                KeyCode::Up => self.selected_index = self.selected_index.saturating_sub(1),
                KeyCode::Down => {
                    let max = self.devices.len().saturating_sub(1);
                    self.selected_index = self.selected_index.saturating_add(1).min(max);
                }
                KeyCode::Enter => {
                    if let Some(dev) = self.devices.get(self.selected_index) {
                        let path = dev.path.clone();
                        self.partition_device = Some(path.clone());
                        // Try reading partition table
                        match PartitionManager::read_table(&path) {
                            Ok((tt, parts)) => {
                                self.partition_table_type = Some(tt);
                                self.partition_table = parts;
                                self.partition_state = PartitionState::ShowTable;
                            }
                            Err(_) => {
                                self.partition_table_type = None;
                                self.partition_table.clear();
                                self.partition_state = PartitionState::ShowTable;
                                self.status_message = Some("No partition table found.".into());
                            }
                        }
                        self.partition_selected = 0;
                    }
                }
                _ => {}
            },
            PartitionState::ShowTable => match key.code {
                KeyCode::Esc => {
                    self.partition_state = PartitionState::SelectDevice;
                    self.selected_index = 0;
                }
                KeyCode::Enter => {
                    self.partition_state = PartitionState::SelectAction;
                    self.partition_selected = 0;
                }
                _ => {}
            },
            PartitionState::SelectAction => {
                // Actions: 0=Add, 1=Delete, 2=Format, 3=CreateTable, 4=Erase
                match key.code {
                    KeyCode::Esc => self.partition_state = PartitionState::ShowTable,
                    KeyCode::Up => {
                        self.partition_selected = self.partition_selected.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        self.partition_selected = self.partition_selected.saturating_add(1).min(4);
                    }
                    KeyCode::Enter => {
                        let action = match self.partition_selected {
                            0 => PartitionActionType::Add,
                            1 => PartitionActionType::Delete,
                            2 => PartitionActionType::Format,
                            3 => PartitionActionType::CreateTable,
                            4 => PartitionActionType::Erase,
                            _ => return false,
                        };
                        self.partition_action = Some(action);
                        self.partition_input.clear();
                        self.partition_input_field = 0;
                        self.partition_error = None;

                        match action {
                            PartitionActionType::Erase | PartitionActionType::CreateTable => {
                                self.partition_state = PartitionState::Confirm;
                            }
                            _ => {
                                self.partition_state = PartitionState::InputParams;
                            }
                        }
                    }
                    _ => {}
                }
            }
            PartitionState::InputParams => match key.code {
                KeyCode::Esc => {
                    self.partition_state = PartitionState::SelectAction;
                    self.partition_input.clear();
                    self.partition_input_field = 0;
                }
                KeyCode::Tab => {
                    let max_fields = match self.partition_action {
                        Some(PartitionActionType::Add) => 2,    // type, size, label
                        Some(PartitionActionType::Delete) => 0, // number
                        Some(PartitionActionType::Format) => 1, // number, type
                        _ => 0,
                    };
                    self.partition_input_field = (self.partition_input_field + 1).min(max_fields);
                }
                KeyCode::BackTab => {
                    self.partition_input_field = self.partition_input_field.saturating_sub(1);
                }
                KeyCode::Char(c) => {
                    self.partition_input.push(c);
                }
                KeyCode::Backspace => {
                    self.partition_input.pop();
                }
                KeyCode::Enter => {
                    self.partition_state = PartitionState::Confirm;
                }
                _ => {}
            },
            PartitionState::Confirm => match key.code {
                KeyCode::Esc => {
                    let prev = match self.partition_action {
                        Some(PartitionActionType::Erase)
                        | Some(PartitionActionType::CreateTable) => PartitionState::SelectAction,
                        _ => PartitionState::InputParams,
                    };
                    self.partition_state = prev;
                }
                KeyCode::Enter => self.execute_partition_action(),
                _ => {}
            },
            PartitionState::Running => {}
            PartitionState::Done | PartitionState::Failed => match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    // Re-read partition table to show updated state
                    if let Some(ref dev) = self.partition_device
                        && let Ok((tt, parts)) = PartitionManager::read_table(dev)
                    {
                        self.partition_table_type = Some(tt);
                        self.partition_table = parts;
                    }
                    self.partition_state = PartitionState::ShowTable;
                    self.partition_error = None;
                    self.partition_progress = None;
                    self.status_message = None;
                }
                _ => {}
            },
        }
        false
    }

    fn execute_partition_action(&mut self) {
        let device = match self.partition_device {
            Some(ref d) => d.clone(),
            None => return,
        };

        match self.partition_action {
            Some(PartitionActionType::CreateTable) => {
                let tt = if self.partition_input.eq_ignore_ascii_case("mbr") {
                    TableType::Mbr
                } else {
                    TableType::Gpt
                };
                match PartitionManager::create_table(&device, tt) {
                    Ok(()) => {
                        self.partition_state = PartitionState::Done;
                        self.status_message = Some("Partition table created.".into());
                        // Refresh
                        if let Ok((tt, parts)) = PartitionManager::read_table(&device) {
                            self.partition_table_type = Some(tt);
                            self.partition_table = parts;
                        }
                    }
                    Err(e) => {
                        self.partition_state = PartitionState::Failed;
                        self.partition_error = Some(format!("{e}"));
                    }
                }
            }
            Some(PartitionActionType::Add) => {
                // Parse input: "ext4 4G label" (space-separated)
                let parts: Vec<&str> = self.partition_input.split_whitespace().collect();
                let fs_str = parts.first().copied().unwrap_or("ext4");
                let size_str = parts.get(1).copied().unwrap_or("remaining");
                let label = parts.get(2).copied();
                let fs = FsType::parse(fs_str);

                match crate::core::partition::parse_size(size_str) {
                    Ok(size_bytes) => {
                        match PartitionManager::add_partition(&device, fs, size_bytes, label, &[]) {
                            Ok(()) => {
                                self.partition_state = PartitionState::Done;
                                self.status_message = Some("Partition added.".into());
                            }
                            Err(e) => {
                                self.partition_state = PartitionState::Failed;
                                self.partition_error = Some(format!("{e}"));
                            }
                        }
                    }
                    Err(e) => {
                        self.partition_state = PartitionState::Failed;
                        self.partition_error = Some(format!("Invalid size: {e}"));
                    }
                }
            }
            Some(PartitionActionType::Delete) => {
                let num: u32 = self.partition_input.trim().parse().unwrap_or(0);
                if num == 0 {
                    self.partition_state = PartitionState::Failed;
                    self.partition_error = Some("Invalid partition number.".into());
                    return;
                }
                match PartitionManager::delete_partition(&device, num) {
                    Ok(()) => {
                        self.partition_state = PartitionState::Done;
                        self.status_message = Some("Partition deleted.".into());
                    }
                    Err(e) => {
                        self.partition_state = PartitionState::Failed;
                        self.partition_error = Some(format!("{e}"));
                    }
                }
            }
            Some(PartitionActionType::Format) => {
                // Parse input: "1 ext4" (number + fs_type)
                let parts: Vec<&str> = self.partition_input.split_whitespace().collect();
                let num: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let fs_str = parts.get(1).copied().unwrap_or("ext4");
                if num == 0 {
                    self.partition_state = PartitionState::Failed;
                    self.partition_error = Some("Invalid partition number.".into());
                    return;
                }
                let fs = FsType::parse(fs_str);
                match PartitionManager::format_partition(&device, num, fs, None) {
                    Ok(()) => {
                        self.partition_state = PartitionState::Done;
                        self.status_message = Some("Partition formatted.".into());
                    }
                    Err(e) => {
                        self.partition_state = PartitionState::Failed;
                        self.partition_error = Some(format!("{e}"));
                    }
                }
            }
            Some(PartitionActionType::Erase) => {
                let (tx, rx) = mpsc::channel(128);
                self.partition_erase_rx = Some(rx);
                self.partition_state = PartitionState::Running;
                self.reset_speed_tracking();

                let dev = device.clone();
                tokio::spawn(async move {
                    let _ = PartitionManager::secure_erase(&dev, EraseMethod::Zero, Some(tx)).await;
                });
            }
            None => {}
        }
    }

    // ── Helpers ───────────────────────────────────────────────────

    fn browser_up(&mut self) {
        if let Some(ref mut fb) = self.file_browser {
            fb.selected = fb.selected.saturating_sub(1);
        }
    }

    fn browser_down(&mut self) {
        if let Some(ref mut fb) = self.file_browser {
            let max = fb.entries.len().saturating_sub(1);
            fb.selected = fb.selected.saturating_add(1).min(max);
        }
    }

    fn home_dir() -> PathBuf {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    }

    fn init_image_browser(&mut self) {
        self.file_browser = Some(FileBrowser::new(
            &Self::home_dir(),
            vec![
                "img".into(),
                "iso".into(),
                "raw".into(),
                "bin".into(),
                "gz".into(),
                "xz".into(),
                "zst".into(),
                "bz2".into(),
                "zip".into(),
            ],
        ));
    }

    fn init_rfb_browser(&mut self) {
        self.file_browser = Some(FileBrowser::new(&Self::home_dir(), vec!["rfb".into()]));
    }

    fn init_all_browser(&mut self) {
        self.file_browser = Some(FileBrowser::new(
            &Self::home_dir(),
            Vec::new(), // show all files
        ));
    }

    fn refresh_devices(&mut self) {
        let enumerator = crate::platform::get_enumerator();
        if let Ok(devs) = enumerator.list_devices(false) {
            self.devices = devs;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
