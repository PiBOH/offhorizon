//! The window: the pages every action lives on. Each button calls [`crate::actions::Session`], so
//! the window and the terminal share one implementation and can never drift apart.

use crate::actions::{Session, Snapshot};
use crate::i18n;
use crate::instances::Edition;
use crate::native::{self, Native};
use crate::others::Found;
use crate::theme::{self, Icon};
use crate::world::{self, Preview, Weather};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Profile,
    Worlds,
    Player,
    Inventory,
    Time,
    Launchers,
    Backups,
}

impl Page {
    const ALL: [Page; 8] = [
        Page::Home,
        Page::Profile,
        Page::Worlds,
        Page::Player,
        Page::Inventory,
        Page::Time,
        Page::Launchers,
        Page::Backups,
    ];

    /// The pages the top bar shows. Profile takes the place Settings has in the official
    /// launcher: its own tab on the right.
    const TABS: [Page; 7] = [
        Page::Home,
        Page::Worlds,
        Page::Player,
        Page::Inventory,
        Page::Time,
        Page::Launchers,
        Page::Backups,
    ];

    fn name(self) -> String {
        i18n::t(match self {
            Page::Home => "nav.home",
            Page::Profile => "nav.profile",
            Page::Worlds => "nav.worlds",
            Page::Player => "nav.player",
            Page::Inventory => "nav.inventory",
            Page::Time => "nav.world",
            Page::Launchers => "nav.launchers",
            Page::Backups => "nav.backups",
        })
    }

    fn icon(self) -> Icon {
        match self {
            Page::Home => Icon::Home,
            Page::Profile => Icon::Person,
            Page::Worlds => Icon::World,
            Page::Player => Icon::Heart,
            Page::Inventory => Icon::Bag,
            Page::Time => Icon::Clock,
            Page::Launchers => Icon::Folder,
            Page::Backups => Icon::Restore,
        }
    }
}

/// What the download thread sends back: its lines as they come, then how it ended.
enum Install {
    Line(String),
    Done(Result<String, String>),
}

pub struct App {
    session: Session,
    page: Page,
    snapshot: Snapshot,
    index: usize,
    name_input: String,
    item_id: String,
    item_count: i32,
    level_input: i32,
    time_input: i64,
    weather_ticks: i32,
    launchers: Vec<Found>,
    versions: Vec<crate::launch::Version>,
    natives: Vec<Native>,
    version_index: usize,
    native_index: usize,
    instance_name: String,
    instance_dir: String,
    instance_portable: bool,
    /// Which game a new instance plays: the Rust client, unless Java is picked.
    edition: Edition,
    /// The instance the sidebar picked: Home plays it.
    instance: Option<String>,
    /// The download running now, if one is.
    installing: Option<std::sync::mpsc::Receiver<Install>>,
    log: Vec<String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let session = Session::open();
        let mut app = Self {
            snapshot: Snapshot::default(),
            launchers: Vec::new(),
            versions: Vec::new(),
            natives: Vec::new(),
            version_index: 0,
            native_index: 0,
            instance_name: String::new(),
            instance_dir: String::new(),
            instance_portable: true,
            edition: Edition::Rust,
            instance: None,
            page: Page::Home,
            index: 0,
            name_input: String::new(),
            item_id: "minecraft:bread".into(),
            item_count: 16,
            level_input: 30,
            time_input: world::TICKS_NOON,
            weather_ticks: 6_000,
            installing: None,
            log: Vec::new(),
            session,
        };
        app.say(format!(
            "{} {} — {}",
            i18n::t("app.name"),
            crate::VERSION,
            i18n::t("portable.telemetry")
        ));
        app.refresh();
        app
    }

    fn say(&mut self, line: impl Into<String>) {
        self.log.push(format!("[{}] {}", clock(), line.into()));
    }

    fn fail(&mut self, line: impl Into<String>) {
        self.log
            .push(format!("[{}] ✗ {}", clock(), line.into()));
    }

    /// Reads the game folders, the worlds and the launchers again.
    fn refresh(&mut self) {
        self.session.rescan();
        self.snapshot = self.session.snapshot();
        self.launchers = self.session.launchers();
        self.versions = self.session.versions();
        self.natives = self.session.natives();
        if self.native_index >= self.natives.len() {
            self.native_index = 0;
        }
        if self.version_index >= self.versions.len() {
            let saved = self.session.portable.get("version");
            self.version_index = saved
                .and_then(|id| self.versions.iter().position(|v| v.id == id))
                .unwrap_or(0);
        }
        if self.index >= self.snapshot.worlds.len() {
            self.index = 0;
        }
        if self.name_input.is_empty() {
            if let Some(name) = crate::launcher::offline_name(&self.session.home()) {
                self.name_input = name;
            }
        }
    }

    fn world(&self) -> Option<world::World> {
        self.snapshot.worlds.get(self.index).cloned()
    }

    fn preview(&self) -> Option<&Preview> {
        self.snapshot.previews.get(self.index)
    }

    /// Runs one action and puts its answer in the log.
    fn carry(&mut self, what: Result<String, String>) {
        match what {
            Ok(said) => self.say(said),
            Err(why) => self.fail(why),
        }
        self.refresh();
    }

    /// Runs one action that needs a world, over the world picked in the window.
    fn carry_world(&mut self, what: impl FnOnce(&mut Session, &world::World) -> Result<String, String>) {
        let Some(world) = self.world() else {
            let why = i18n::t("status.no_world");
            self.fail(why);
            return;
        };
        let result = what(&mut self.session, &world);
        self.carry(result);
    }
}

fn clock() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let day = secs % 86_400;
    format!("{:02}:{:02}:{:02}", day / 3600, (day / 60) % 60, day % 60)
}

/// A label when there is a value, the reason there is not when there is none.
fn pinned(value: Option<String>, missing: String) -> String {
    value.unwrap_or(missing)
}

fn bytes(size: u64) -> String {
    if size >= 1_048_576 {
        format!("{:.1} MiB", size as f64 / 1_048_576.0)
    } else if size >= 1024 {
        format!("{:.1} KiB", size as f64 / 1024.0)
    } else {
        format!("{size} B")
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_install(ctx);
        // The top bar, from the official launcher: the pages as tabs over the page, then who
        // plays and Settings. No plate: the page shows through, under a 1 px rule.
        egui::TopBottomPanel::top("top_bar")
            .exact_height(theme::TOP_BAR)
            .frame(
                egui::Frame::none()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(theme::INSET, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    for page in Page::TABS {
                        if theme::tab(ui, &page.name(), self.page == page).clicked() {
                            self.page = page;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::tab(ui, &Page::Profile.name(), self.page == Page::Profile)
                            .clicked()
                        {
                            self.page = Page::Profile;
                        }
                        let native = i18n::LANGUAGES
                            .iter()
                            .find(|(code, _)| *code == self.session.lang.as_str())
                            .map(|(_, name)| *name)
                            .unwrap_or("English");
                        egui::ComboBox::from_id_salt("lang")
                            .selected_text(native)
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for (code, name) in i18n::LANGUAGES {
                                    let picked = self.session.lang == *code;
                                    if ui.selectable_label(picked, *name).clicked() {
                                        let code = code.to_string();
                                        let result = self.session.set_language(&code);
                                        self.carry(result);
                                    }
                                }
                            });
                        if ui.button(i18n::t("common.rescan")).clicked() {
                            self.refresh();
                            self.say(i18n::tf(
                                "status.done",
                                &[("what", i18n::t("common.rescan"))],
                            ));
                        }
                    });
                });
            });

        egui::TopBottomPanel::top("top_rule")
            .exact_height(1.0)
            .frame(egui::Frame::none().fill(theme::LINE))
            .show(ctx, |_| {});

        // The sidebar, from the official launcher: the wordmark, then the instances, each a row
        // that picks it; Home's own Play button starts the one picked.
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(theme::SIDEBAR)
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(12.0, 14.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    crate::logo::draw(ui, 24.0);
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(i18n::t("app.name"))
                            .size(16.0)
                            .color(theme::TEXT)
                            .strong(),
                    );
                });
                ui.label(
                    egui::RichText::new(crate::VERSION)
                        .size(11.5)
                        .color(theme::FAINT),
                );
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(i18n::t("home.instances"))
                            .size(12.0)
                            .color(theme::DIM)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::icon_button(ui, Icon::Play, theme::ACCENT).clicked() {
                            self.page = Page::Home;
                        }
                    });
                });
                ui.add_space(6.0);
                let instances = self.session.instances();
                let portable_home = self.session.home();
                if instances.is_empty() {
                    ui.label(
                        egui::RichText::new(i18n::t("home.no_instances"))
                            .size(12.5)
                            .color(theme::DIM),
                    );
                    ui.add_space(6.0);
                }
                let mut picked: Option<String> = None;
                for instance in &instances {
                    let sub = if instance.game == portable_home {
                        format!("{} · {}", instance.edition.name(), instance.version)
                    } else {
                        format!(
                            "{} · {} · {}",
                            instance.edition.name(),
                            instance.version,
                            instance.game.display()
                        )
                    };
                    let current = self.instance.as_deref() == Some(instance.name.as_str());
                    if theme::sidebar_row(ui, &instance.name, &sub, current).clicked() {
                        picked = Some(instance.name.clone());
                    }
                    ui.add_space(2.0);
                }
                if let Some(name) = picked {
                    self.instance = Some(name);
                    self.page = Page::Home;
                }
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new(i18n::t("portable.minecraft"))
                        .size(11.5)
                        .color(theme::FAINT),
                );
                ui.label(
                    egui::RichText::new(self.session.home().display().to_string())
                        .size(11.5)
                        .color(theme::DIM),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(i18n::t("portable.telemetry_off"))
                        .size(11.5)
                        .color(theme::GREEN),
                );
            });

        egui::TopBottomPanel::bottom("log")
            .resizable(true)
            .default_height(120.0)
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(theme::label(i18n::t("common.open")));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(i18n::t("common.clear")).clicked() {
                            self.log.clear();
                        }
                    });
                });
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for line in &self.log {
                            let color = if line.contains('✗') {
                                theme::RED
                            } else {
                                theme::DIM
                            };
                            ui.label(
                                egui::RichText::new(line)
                                    .monospace()
                                    .size(12.0)
                                    .color(color),
                            );
                        }
                    });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::same(theme::INSET)),
            )
            .show(ctx, |ui| match self.page {
                Page::Home => self.home_page(ui),
                Page::Profile => self.profile_page(ui),
                Page::Worlds => self.worlds_page(ui),
                Page::Player => self.player_page(ui),
                Page::Inventory => self.inventory_page(ui),
                Page::Time => self.time_page(ui),
                Page::Launchers => self.launchers_page(ui),
                Page::Backups => self.backups_page(ui),
            });
    }
}

impl App {
    /// Home, from the official launcher: the instance picked on the left, its name and Play,
    /// then the player, a new instance, the versions and the last world.
    fn home_page(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let versions = self.versions.clone();
                let natives = self.natives.clone();
                let instances = self.session.instances();
                let portable_home = self.session.home();
                let chosen = self
                    .instance
                    .as_deref()
                    .and_then(|name| instances.iter().find(|known| known.name == name))
                    .cloned()
                    .or_else(|| instances.first().cloned());
                if let Some(instance) = &chosen {
                    self.instance = Some(instance.name.clone());
                }

                // The head: the instance's name, what it plays, and who plays it.
                let head = chosen
                    .as_ref()
                    .map(|instance| instance.name.clone())
                    .unwrap_or_else(|| i18n::t("app.name"));
                ui.label(
                    egui::RichText::new(head)
                        .size(30.0)
                        .color(theme::TEXT)
                        .strong(),
                );
                ui.add_space(4.0);
                let sub = match &chosen {
                    Some(instance) => format!(
                        "{} · {} · {}",
                        instance.edition.name(),
                        instance.version,
                        if instance.game == portable_home {
                            i18n::t("home.portable")
                        } else {
                            instance.game.display().to_string()
                        }
                    ),
                    None => i18n::t("home.tagline"),
                };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(sub).size(13.5).color(theme::DIM));
                    ui.label(egui::RichText::new("·").size(13.5).color(theme::FAINT));
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            i18n::t("home.player"),
                            self.session.player_name()
                        ))
                        .size(13.5)
                        .color(theme::DIM),
                    );
                });

                // The play group: Play, and the version it starts.
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    if theme::play_button(ui, &i18n::t("home.play")).clicked() {
                        match &chosen {
                            Some(instance) => {
                                let name = instance.name.clone();
                                let result = self.session.play_instance(&name);
                                self.carry(result);
                            }
                            None => self.play(),
                        }
                    }
                    ui.add_space(10.0);
                    egui::Frame::none()
                        .fill(theme::PANEL)
                        .rounding(theme::RADIUS_CONTROL)
                        .inner_margin(egui::Margin::symmetric(16.0, 11.0))
                        .show(ui, |ui| {
                            ui.label(theme::label(i18n::t("home.versions")));
                            let playing = match &chosen {
                                Some(instance) => {
                                    format!("{} · {}", instance.edition.name(), instance.version)
                                }
                                None => match self.edition {
                                    Edition::Rust => natives.get(self.native_index).map(|client| {
                                        format!("{} · {}", i18n::t("home.rust"), client.tag)
                                    }),
                                    Edition::Java => versions.get(self.version_index).map(|version| {
                                        format!("{} · {}", i18n::t("home.java"), version.id)
                                    }),
                                }
                                .unwrap_or_else(|| "—".into()),
                            };
                            ui.label(
                                egui::RichText::new(playing)
                                    .size(14.0)
                                    .color(theme::TEXT),
                            );
                        });
                });

                // The player's name: every launch hands it to the game.
                ui.add_space(20.0);
                theme::plate().show(ui, |ui| {
                    theme::group_title(ui, i18n::t("home.player"));
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.name_input)
                                .hint_text("Steve")
                                .desired_width(200.0),
                        );
                        if ui
                            .button(theme::action(&i18n::t("profile.set_name")))
                            .clicked()
                        {
                            let name = self.name_input.clone();
                            let result = self.session.set_name(&name);
                            self.carry(result);
                        }
                    });
                });

                // A new instance: a version, a name, and where it plays.
                ui.add_space(12.0);
                theme::plate().show(ui, |ui| {
                    theme::group_title(ui, i18n::t("home.new_instance"));
                    ui.add_space(6.0);
                    theme::field_row(ui, i18n::t("home.edition"), |ui| {
                        for edition in [Edition::Rust, Edition::Java] {
                            if ui
                                .selectable_label(self.edition == edition, edition.name())
                                .clicked()
                            {
                                self.edition = edition;
                            }
                        }
                    });
                    let picked = match self.edition {
                        Edition::Rust => natives.get(self.native_index).map(|c| c.tag.clone()),
                        Edition::Java => versions.get(self.version_index).map(|v| v.id.clone()),
                    };
                    let selected = match self.edition {
                        Edition::Rust => pinned(
                            natives
                                .get(self.native_index)
                                .map(|client| {
                                    format!(
                                        "{} · {}",
                                        client.tag,
                                        native::minecraft_of(client)
                                    )
                                }),
                            i18n::t("home.rust_missing"),
                        ),
                        Edition::Java => versions
                            .get(self.version_index)
                            .map(|version| format!("{} · {}", version.id, version.loader))
                            .unwrap_or_else(|| "—".into()),
                    };
                    theme::field_row(ui, i18n::t("home.versions"), |ui| {
                        egui::ComboBox::from_id_salt("new_instance_version")
                            .selected_text(selected)
                            .width(240.0)
                            .show_ui(ui, |ui| match self.edition {
                                Edition::Rust => {
                                    for (index, client) in natives.iter().enumerate() {
                                        let label = format!(
                                            "{} · {}",
                                            client.tag,
                                            native::minecraft_of(client)
                                        );
                                        if ui
                                            .selectable_label(self.native_index == index, label)
                                            .clicked()
                                        {
                                            self.native_index = index;
                                        }
                                    }
                                }
                                Edition::Java => {
                                    for (index, version) in versions.iter().enumerate() {
                                        let label = format!("{} · {}", version.id, version.loader);
                                        if ui
                                            .selectable_label(self.version_index == index, label)
                                            .clicked()
                                        {
                                            self.version_index = index;
                                        }
                                    }
                                }
                            });
                    });
                    theme::field_row(ui, i18n::t("home.instance_name"), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.instance_name)
                                .hint_text(picked.clone().unwrap_or_default())
                                .desired_width(200.0),
                        );
                    });
                    theme::field_row(ui, i18n::t("home.portable"), |ui| {
                        ui.checkbox(&mut self.instance_portable, "");
                        if !self.instance_portable {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.instance_dir)
                                    .hint_text(i18n::t("home.folder"))
                                    .desired_width(260.0),
                            );
                        }
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(theme::action(&i18n::t("home.create"))).clicked() {
                            let edition = self.edition;
                            let version = match edition {
                                Edition::Rust => {
                                    natives.get(self.native_index).map(|client| client.tag.clone())
                                }
                                Edition::Java => {
                                    versions.get(self.version_index).map(|known| known.id.clone())
                                }
                            };
                            if let Some(version) = version {
                                let name = if self.instance_name.trim().is_empty() {
                                    version.clone()
                                } else {
                                    self.instance_name.clone()
                                };
                                let dir = (!self.instance_portable
                                    && !self.instance_dir.trim().is_empty())
                                .then(|| std::path::PathBuf::from(self.instance_dir.trim()));
                                let result =
                                    self.session.create_instance(&name, &version, dir, edition);
                                self.carry(result);
                            }
                        }
                    });
                });

                // The instances, the one picked first.
                ui.add_space(12.0);
                theme::plate().show(ui, |ui| {
                    theme::group_title(ui, i18n::t("home.instances"));
                    ui.add_space(6.0);
                    if instances.is_empty() {
                        ui.label(theme::label(i18n::t("home.no_instances")));
                    }
                    let mut start: Option<String> = None;
                    let mut remove: Option<String> = None;
                    for instance in &instances {
                        let sub = if instance.game == portable_home {
                            format!("{} · {}", instance.edition.name(), instance.version)
                        } else {
                            format!(
                                "{} · {} · {}",
                                instance.edition.name(),
                                instance.version,
                                instance.game.display()
                            )
                        };
                        let current = self.instance.as_deref() == Some(instance.name.as_str());
                        if theme::list_row(ui, &instance.name, &sub, current).clicked() {
                            self.instance = Some(instance.name.clone());
                        }
                        ui.add_space(2.0);
                    }
                    if let Some(instance) = &chosen {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui
                                .button(theme::action(&i18n::t("home.play")))
                                .clicked()
                            {
                                start = Some(instance.name.clone());
                            }
                            if ui.small_button(i18n::t("common.remove")).clicked() {
                                remove = Some(instance.name.clone());
                            }
                        });
                    }
                    if let Some(name) = start {
                        let result = self.session.play_instance(&name);
                        self.carry(result);
                    }
                    if let Some(name) = remove {
                        let result = self.session.remove_instance(&name);
                        self.carry(result);
                    }
                });

                // Every client and version found here, the Rust clients first.
                ui.add_space(12.0);
                theme::plate().show(ui, |ui| {
                    theme::group_title(ui, i18n::t("home.rust"));
                    ui.add_space(6.0);
                    if natives.is_empty() {
                        ui.label(theme::note(i18n::t("home.rust_missing"), false));
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let busy = self.installing.is_some();
                        if ui
                            .add_enabled(
                                !busy,
                                egui::Button::new(theme::action(&i18n::t("home.rust_download"))),
                            )
                            .clicked()
                        {
                            self.start_install();
                        }
                        if busy {
                            ui.spinner();
                        }
                    });
                    ui.add_space(6.0);
                    let mut pick_native: Option<usize> = None;
                    for (index, client) in natives.iter().enumerate() {
                        let sub = format!(
                            "{} · {}",
                            native::minecraft_of(client),
                            client.resources.display()
                        );
                        if theme::list_row(
                            ui,
                            &client.tag,
                            &sub,
                            self.edition == Edition::Rust && index == self.native_index,
                        )
                        .clicked()
                        {
                            pick_native = Some(index);
                        }
                        ui.add_space(2.0);
                    }
                    if let Some(index) = pick_native {
                        self.native_index = index;
                        self.edition = Edition::Rust;
                    }
                    ui.add_space(10.0);
                    theme::group_title(ui, i18n::t("home.java"));
                    ui.add_space(6.0);
                    if versions.is_empty() {
                        ui.label(theme::note(i18n::t("home.no_version"), false));
                        if ui.button(theme::action(&i18n::t("nav.launchers"))).clicked() {
                            self.page = Page::Launchers;
                        }
                    }
                    let mut pick: Option<usize> = None;
                    for (index, version) in versions.iter().enumerate() {
                        let sub = format!(
                            "{} · {} · {}",
                            version.loader,
                            version.release_time,
                            version.game.display()
                        );
                        if theme::list_row(
                            ui,
                            &version.id,
                            &sub,
                            self.edition == Edition::Java && index == self.version_index,
                        )
                        .clicked()
                        {
                            pick = Some(index);
                        }
                        ui.add_space(2.0);
                    }
                    if let Some(index) = pick {
                        self.version_index = index;
                        self.edition = Edition::Java;
                    }
                });

                // The world played last, with what the player has in it.
                if let Some(preview) = self.snapshot.previews.first().cloned() {
                    ui.add_space(12.0);
                    theme::plate().show(ui, |ui| {
                        theme::group_title(ui, i18n::t("home.last_world"));
                        ui.horizontal(|ui| {
                            theme::icon(ui, Icon::World, theme::ACCENT);
                            ui.label(theme::value(preview.label.clone()));
                        });
                        ui.add_space(4.0);
                        ui.add(
                            egui::ProgressBar::new(preview.health_fraction())
                                .desired_width(240.0)
                                .text(format!("{:.0}/20", preview.health.unwrap_or(0.0))),
                        );
                        ui.label(theme::label(format!(
                            "{} x{} · {} {} · {} {}",
                            i18n::t("worlds.food"),
                            preview.food.unwrap_or(0),
                            i18n::t("worlds.items"),
                            preview.items.len(),
                            i18n::t("worlds.level"),
                            preview.xp_level.unwrap_or(0)
                        )));
                        ui.add_space(6.0);
                        if ui.button(theme::action(&i18n::t("nav.worlds"))).clicked() {
                            self.page = Page::Worlds;
                        }
                    });
                }
            });
    }

    /// Downloads the Horizon client in the background: the window keeps answering, and the log
    /// fills with the download's lines as they come.
    fn start_install(&mut self) {
        if self.installing.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.installing = Some(rx);
        std::thread::spawn(move || {
            let mut session = Session::open();
            let mut said = |line: String| {
                let _ = tx.send(Install::Line(line));
            };
            let result = session.install_native(&mut said);
            let _ = tx.send(Install::Done(result));
        });
    }

    /// Takes what the download thread sent: its lines into the log, its answer into the pages.
    fn poll_install(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.installing.take() else {
            return;
        };
        let mut running = true;
        loop {
            match rx.try_recv() {
                Ok(Install::Line(line)) => self.say(line),
                Ok(Install::Done(result)) => {
                    self.carry(result);
                    running = false;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    running = false;
                    break;
                }
            }
        }
        if running {
            self.installing = Some(rx);
            ctx.request_repaint();
        }
    }

    /// Starts what the home page shows, as the offline name: the Rust client when there is one and
    /// Rust is the edition picked, the Java version otherwise.
    fn play(&mut self) {
        let result = match self.edition {
            Edition::Rust => {
                let tag = self.natives.get(self.native_index).map(|c| c.tag.clone());
                match tag {
                    Some(tag) => self.session.play_native(Some(tag.as_str())),
                    None => Err(i18n::t("home.rust_missing")),
                }
            }
            Edition::Java => {
                let id = self.versions.get(self.version_index).map(|v| v.id.clone());
                self.session.play_edition(id.as_deref(), Some(Edition::Java))
            }
        };
        self.carry(result);
    }

    fn profile_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title(&i18n::t("profile.title")));
        ui.label(theme::label(i18n::t("profile.blurb")));
        ui.add_space(12.0);
        let home = self.session.home();
        let file = crate::launcher::player_name_file(&home);
        let current = crate::launcher::offline_name(&home).unwrap_or_else(|| "—".into());
        ui.horizontal(|ui| {
            ui.label(theme::label(i18n::t("profile.file")));
            ui.label(theme::value(file.display().to_string()));
        });
        ui.horizontal(|ui| {
            ui.label(theme::label(i18n::t("profile.name_now")));
            ui.label(theme::value(current));
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(theme::label(i18n::t("profile.offline_name")));
            ui.add(
                egui::TextEdit::singleline(&mut self.name_input)
                    .hint_text("Steve")
                    .desired_width(180.0),
            );
            if ui.button(theme::action(&i18n::t("profile.set_name"))).clicked() {
                let name = self.name_input.clone();
                let result = self.session.set_name(&name);
                self.carry(result);
            }
        });

        ui.add_space(18.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(theme::label(i18n::t("portable.title")));
        ui.label(theme::label(i18n::t("portable.dirs")));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(theme::action(&i18n::t("portable.point"))).clicked() {
                match self.session.point_launcher() {
                    Ok(change) => {
                        self.say(format!(
                            "{} → {}",
                            change.settings.display(),
                            self.session.home().display()
                        ));
                        for copy in &change.copies {
                            self.say(format!("copy: {}", copy.display()));
                        }
                    }
                    Err(why) => self.fail(why),
                }
                self.refresh();
            }
            if ui.button(i18n::t("portable.unpoint")).clicked() {
                match self.session.undo_launcher() {
                    Ok(file) => self.say(format!("{} {}", i18n::t("portable.unpoint"), file.display())),
                    Err(why) => self.fail(why),
                }
                self.refresh();
            }
        });
    }

    fn worlds_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title(&i18n::t("worlds.title")));
        ui.add_space(6.0);
        if self.snapshot.previews.is_empty() {
            ui.label(theme::note(i18n::t("worlds.none"), false));
            return;
        }
        let previews = self.snapshot.previews.clone();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, preview) in previews.iter().enumerate() {
                    let picked = index == self.index;
                    egui::Frame::none()
                        .fill(if picked { theme::PANEL_HI } else { theme::PANEL })
                        .stroke(egui::Stroke::new(
                            1.0,
                            if picked { theme::ACCENT } else { theme::LINE },
                        ))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                theme::icon(ui, Icon::World, theme::ACCENT);
                                ui.label(theme::value(preview.label.clone()));
                                ui.label(theme::label(bytes(preview.bytes)));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button(i18n::t("common.open")).clicked() {
                                            self.index = index;
                                            self.page = Page::Player;
                                        }
                                    },
                                );
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(theme::label(i18n::t("worlds.health")));
                                ui.add(
                                    egui::ProgressBar::new(preview.health_fraction())
                                        .desired_width(120.0)
                                        .text(format!(
                                            "{:.0}/20",
                                            preview.health.unwrap_or(0.0)
                                        )),
                                );
                                ui.label(theme::label(format!(
                                    "{} {}/20",
                                    i18n::t("worlds.food"),
                                    preview.food.unwrap_or(0)
                                )));
                                ui.label(theme::label(format!(
                                    "{} {}",
                                    i18n::t("worlds.level"),
                                    preview.xp_level.unwrap_or(0)
                                )));
                            });
                            if let Some(error) = &preview.error {
                                ui.label(theme::note(error.clone(), false));
                            } else {
                                ui.label(theme::label(
                                    preview
                                        .items
                                        .iter()
                                        .map(|item| format!("{} x{}", short(&item.id), item.count))
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                ));
                            }
                        });
                    ui.add_space(8.0);
                }
            });
    }

    fn player_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title(&i18n::t("player.title")));
        ui.add_space(6.0);
        self.world_picker(ui);
        ui.add_space(12.0);
        let Some(stats) = self
            .preview()
            .map(|p| (p.health, p.food, p.xp_level, p.error.clone()))
        else {
            ui.label(theme::note(i18n::t("status.no_world"), false));
            return;
        };
        if let Some(error) = stats.3 {
            ui.label(theme::note(error, false));
        }
        egui::Grid::new("stats")
            .num_columns(3)
            .spacing([28.0, 10.0])
            .show(ui, |ui| {
                ui.label(theme::label(i18n::t("worlds.health")));
                ui.label(theme::label(i18n::t("worlds.food")));
                ui.label(theme::label(i18n::t("worlds.level")));
                ui.end_row();
                ui.label(theme::value(format!("{:.1} / 20", stats.0.unwrap_or(0.0))));
                ui.label(theme::value(format!("{} / 20", stats.1.unwrap_or(0))));
                ui.label(theme::value(stats.2.unwrap_or(0).to_string()));
                ui.end_row();
            });
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [150.0, 36.0],
                    egui::Button::new(theme::action(&format!("♥  {}", i18n::t("player.heal")))),
                )
                .clicked()
            {
                self.carry_world(|session, world| session.heal(world));
            }
            if ui
                .add_sized(
                    [150.0, 36.0],
                    egui::Button::new(theme::action(&format!("🍗  {}", i18n::t("player.feed")))),
                )
                .clicked()
            {
                self.carry_world(|session, world| session.feed(world));
            }
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(theme::label(i18n::t("player.level")));
            ui.add(egui::DragValue::new(&mut self.level_input).range(0..=30_000));
            if ui.button(i18n::t("common.set")).clicked() {
                let level = self.level_input;
                self.carry_world(|session, world| session.set_level(world, level));
            }
        });
    }

    fn world_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(theme::label(i18n::t("nav.worlds")));
            let selected = self
                .preview()
                .map(|p| p.label.clone())
                .unwrap_or_else(|| "—".into());
            egui::ComboBox::from_id_salt("world")
                .selected_text(selected)
                .width(280.0)
                .show_ui(ui, |ui| {
                    let labels: Vec<String> =
                        self.snapshot.previews.iter().map(|p| p.label.clone()).collect();
                    for (index, label) in labels.iter().enumerate() {
                        if ui
                            .selectable_label(self.index == index, label)
                            .clicked()
                        {
                            self.index = index;
                        }
                    }
                });
            if ui.button(i18n::t("common.reload")).clicked() {
                self.refresh();
            }
        });
    }

    fn inventory_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title(&i18n::t("inv.title")));
        ui.add_space(6.0);
        self.world_picker(ui);
        ui.add_space(10.0);
        let Some(preview) = self.preview().cloned() else {
            ui.label(theme::note(i18n::t("status.no_world"), false));
            return;
        };
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("items")
                    .num_columns(3)
                    .striped(true)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(theme::label("SLOT"));
                        ui.label(theme::label(i18n::t("worlds.items")));
                        ui.label(theme::label(i18n::t("inv.count")));
                        ui.end_row();
                        let mut removed = None;
                        for item in &preview.items {
                            ui.label(theme::value(item.slot.to_string()));
                            ui.label(theme::value(item.id.clone()));
                            ui.horizontal(|ui| {
                                ui.label(theme::value(item.count.to_string()));
                                if ui.small_button(i18n::t("common.remove")).clicked() {
                                    removed = Some(item.slot);
                                }
                            });
                            ui.end_row();
                        }
                        if preview.items.is_empty() {
                            ui.label(theme::label(i18n::t("inv.empty")));
                            ui.end_row();
                        }
                        if let Some(slot) = removed {
                            self.carry_world(|session, world| session.remove_slot(world, slot));
                        }
                    });
            });
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(theme::label(i18n::t("inv.add")));
            ui.add(
                egui::TextEdit::singleline(&mut self.item_id)
                    .hint_text("minecraft:bread")
                    .desired_width(260.0),
            );
            ui.add(egui::DragValue::new(&mut self.item_count).range(1..=64));
            if ui.button(theme::action(&i18n::t("common.add"))).clicked() {
                let id = self.item_id.clone();
                let count = self.item_count;
                self.carry_world(|session, world| session.add_item(world, &id, count));
            }
        });
        ui.add_space(6.0);
        ui.label(theme::label(i18n::t("inv.hint")));
    }

    fn time_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title(&i18n::t("world.title")));
        ui.add_space(6.0);
        self.world_picker(ui);
        ui.add_space(12.0);
        let Some(preview) = self.preview().cloned() else {
            ui.label(theme::note(i18n::t("status.no_world"), false));
            return;
        };
        let ticks = world::load(&preview.level)
            .ok()
            .and_then(|root| world::time_of_day(&root));
        ui.horizontal(|ui| {
            ui.label(theme::label(i18n::t("world.time")));
            ui.label(theme::value(match ticks {
                Some(ticks) => format!("{ticks} {}", i18n::t("world.ticks")),
                None => "—".into(),
            }));
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (name, ticks) in [
                (i18n::t("world.day"), world::TICKS_DAY),
                (i18n::t("world.noon"), world::TICKS_NOON),
                (i18n::t("world.night"), world::TICKS_NIGHT),
                (i18n::t("world.midnight"), world::TICKS_MIDNIGHT),
            ] {
                if ui.button(name).clicked() {
                    self.carry_world(|session, world| session.set_time(world, ticks));
                }
            }
            ui.add(egui::DragValue::new(&mut self.time_input).range(0..=23_999));
            if ui.button(i18n::t("world.set_time")).clicked() {
                let ticks = self.time_input;
                self.carry_world(|session, world| session.set_time(world, ticks));
            }
        });
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(theme::label(i18n::t("world.weather")));
        ui.horizontal(|ui| {
            for (name, weather) in [
                (i18n::t("world.clear"), Weather::Clear),
                (i18n::t("world.rain"), Weather::Rain),
                (i18n::t("world.thunder"), Weather::Thunder),
            ] {
                if ui.button(name).clicked() {
                    let ticks = self.weather_ticks;
                    self.carry_world(|session, world| session.set_weather(world, weather, ticks));
                }
            }
            ui.label(theme::label(i18n::t("world.ticks")));
            ui.add(egui::DragValue::new(&mut self.weather_ticks).range(1..=1_000_000));
        });
    }

    fn launchers_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title(&i18n::t("launchers.title")));
        ui.add_space(6.0);
        if self.launchers.is_empty() {
            ui.label(theme::note(i18n::t("launchers.none"), false));
        }
        let launchers = self.launchers.clone();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for install in &launchers {
                    ui.horizontal(|ui| {
                        theme::icon(ui, Icon::Folder, theme::ACCENT);
                        ui.label(theme::value(install.name));
                        ui.label(theme::label(install.kind));
                        ui.label(theme::label(format!(
                            "{}: {}",
                            i18n::t("launchers.worlds"),
                            install.worlds
                        )));
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button(i18n::t("launchers.import")).clicked() {
                                    let id = install.id.to_string();
                                    let result = self.session.import(&id);
                                    self.carry(result);
                                }
                            },
                        );
                    });
                    ui.label(theme::label(install.game.display().to_string()));
                    ui.add_space(6.0);
                }
            });
    }

    fn backups_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title(&i18n::t("backups.title")));
        ui.label(theme::label(self.session.store.root.display().to_string()));
        ui.add_space(10.0);
        if ui.button(i18n::t("common.reload")).clicked() {
            self.refresh();
        }
        ui.add_space(10.0);
        let entries = self.session.backups().to_vec();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("backups")
                    .num_columns(3)
                    .striped(true)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(theme::label("WHEN"));
                        ui.label(theme::label(i18n::t("portable.launcher")));
                        ui.label("");
                        ui.end_row();
                        let mut restore = None;
                        for entry in &entries {
                            ui.label(theme::value(entry.id.clone()));
                            ui.label(theme::value(entry.tool.clone()))
                                .on_hover_text(entry.original.display().to_string());
                            if ui.small_button(i18n::t("common.restore")).clicked() {
                                restore = Some(entry.id.clone());
                            }
                            ui.end_row();
                        }
                        if entries.is_empty() {
                            ui.label(theme::label(i18n::t("backups.none")));
                            ui.end_row();
                        }
                        if let Some(id) = restore {
                            let result = self.session.restore(&id);
                            self.carry(result);
                        }
                    });
            });
    }
}

fn short(id: &str) -> &str {
    id.strip_prefix("minecraft:").unwrap_or(id)
}
