//! The window: what the tool found on this computer on the left, the page it is on in the middle,
//! and what it did at the bottom.

use crate::{items, launcher, nbt, store::Store, theme, world};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Profile,
    Player,
    Inventory,
    Backups,
}

impl Tab {
    const ALL: [Tab; 4] = [Tab::Profile, Tab::Player, Tab::Inventory, Tab::Backups];

    fn name(self) -> &'static str {
        match self {
            Tab::Profile => "Profile",
            Tab::Player => "Player",
            Tab::Inventory => "Inventory",
            Tab::Backups => "Backups",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Tab::Profile => "◈",
            Tab::Player => "♥",
            Tab::Inventory => "▤",
            Tab::Backups => "⟲",
        }
    }
}

pub struct App {
    tab: Tab,
    layout: launcher::Layout,
    home_index: usize,
    worlds: Vec<world::World>,
    world_index: usize,
    root: Option<nbt::Root>,
    stats: Option<world::Stats>,
    name_input: String,
    item_id: String,
    item_count: i32,
    level_input: i32,
    store: Store,
    log: Vec<String>,
    load_requested: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let layout = launcher::detect();
        let mut app = Self {
            tab: Tab::Profile,
            home_index: 0,
            layout,
            worlds: Vec::new(),
            world_index: 0,
            root: None,
            stats: None,
            name_input: String::new(),
            item_id: "minecraft:bread".into(),
            item_count: 16,
            level_input: 30,
            store: Store::open(),
            log: Vec::new(),
            load_requested: false,
        };
        app.say(format!("Offline Patch {}", crate::VERSION));
        if let Some(home) = app.home() {
            app.say(format!("Game folder: {}", home.display()));
        } else {
            app.say("No game folder found: start the game once, or set HORIZON_HOME.");
        }
        if let Some(launcher) = app.layout.launcher.clone() {
            app.say(format!("Launcher data: {}", launcher.display()));
        }
        app.reload_worlds();
        app
    }

    fn home(&self) -> Option<PathBuf> {
        self.layout.homes.get(self.home_index).cloned()
    }

    /// A line in the log at the bottom.
    fn say(&mut self, line: impl Into<String>) {
        self.log.push(format!("[{}] {}", clock(), line.into()));
    }

    fn fail(&mut self, line: impl Into<String>) {
        self.say(format!("✗ {}", line.into()));
    }

    fn rescan(&mut self) {
        self.layout = launcher::detect();
        self.home_index = self.home_index.min(self.layout.homes.len().saturating_sub(1));
        self.say(format!(
            "Found {} game folder(s){}",
            self.layout.homes.len(),
            if self.layout.launcher.is_some() {
                " and a launcher"
            } else {
                ""
            }
        ));
        self.reload_worlds();
    }

    fn reload_worlds(&mut self) {
        self.worlds = self
            .home()
            .map(|home| world::worlds(&home))
            .unwrap_or_default();
        self.world_index = 0;
        self.say(format!("{} world(s) under saves/", self.worlds.len()));
        if self.worlds.is_empty() {
            self.root = None;
            self.stats = None;
        } else {
            self.load_world();
        }
        let name = self.home().and_then(|h| launcher::offline_name(&h));
        if let Some(name) = name {
            self.name_input = name.clone();
            self.say(format!("Offline player name: {name}"));
        }
    }

    fn load_world(&mut self) {
        let Some(world) = self.worlds.get(self.world_index).cloned() else {
            return;
        };
        match world::load(&world.level) {
            Ok(root) => match world::stats(&root) {
                Ok(stats) => {
                    self.say(format!("Loaded “{}”", world.name));
                    self.stats = Some(stats);
                    self.root = Some(root);
                }
                Err(e) => {
                    self.fail(e);
                    self.root = Some(root);
                    self.stats = None;
                }
            },
            Err(e) => {
                self.fail(e);
                self.root = None;
                self.stats = None;
            }
        }
    }

    /// Copies the save, changes it and writes it back: the one path every edit takes.
    fn commit(&mut self, what: &str, edit: impl FnOnce(&mut nbt::Root) -> Result<String, String>) {
        let Some(world) = self.worlds.get(self.world_index).cloned() else {
            self.fail("Pick a world first.");
            return;
        };
        let Some(mut root) = self.root.take() else {
            self.fail("No save is loaded.");
            return;
        };
        let said = match edit(&mut root) {
            Ok(said) => said,
            Err(e) => {
                self.root = Some(root);
                self.fail(format!("{what}: {e}"));
                return;
            }
        };
        if let Err(e) = self.store.backup(&world.level, what) {
            self.root = Some(root);
            self.fail(format!("No backup, so nothing was written: {e}"));
            return;
        }
        if let Err(e) = world::save(&world.level, &root) {
            self.root = Some(root);
            self.fail(e);
            return;
        }
        self.stats = world::stats(&root).ok();
        self.root = Some(root);
        self.say(format!("{what}: {said} (backup kept)"));
    }

    fn set_name(&mut self) {
        let Some(home) = self.home() else {
            self.fail("Pick a game folder first.");
            return;
        };
        let name = self.name_input.clone();
        let mut store = std::mem::replace(&mut self.store, Store::open());
        let result = launcher::set_offline_name(&mut store, &home, &name);
        self.store = store;
        match result {
            Ok(file) => {
                self.say(format!("Offline name {name} written to {}", file.display()));
                let _ = home;
            }
            Err(e) => self.fail(e),
        }
    }

    fn remove_item(&mut self, slot: i32) {
        self.commit("Remove item", |root| world::remove_slot(root, slot));
    }

    fn add_item(&mut self) {
        let id = self.item_id.clone();
        let count = self.item_count;
        self.commit("Add item", |root| world::add_item(root, &id, count));
    }

    fn heal(&mut self) {
        self.commit("Heal", world::heal);
    }

    fn feed(&mut self) {
        self.commit("Feed", world::feed);
    }

    fn set_level(&mut self) {
        let level = self.level_input;
        self.commit("Set level", |root| world::set_level(root, level));
    }

    fn restore(&mut self, id: String) {
        match self.store.restore(&id) {
            Ok(file) => {
                self.say(format!("Restored {}", file.display()));
                self.load_world();
            }
            Err(e) => self.fail(e),
        }
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("head")
            .frame(egui::Frame::none().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(16.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("OFFLINE PATCH").size(20.0).color(theme::AMBER).strong());
                    ui.label(theme::label(crate::VERSION));
                    ui.separator();
                    match &self.layout.launcher {
                        Some(path) => {
                            ui.label(theme::label(format!("launcher: {}", path.display())));
                        }
                        None => {
                            ui.label(theme::note("no Horizon launcher data folder found", false));
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Rescan").clicked() {
                            self.rescan();
                        }
                    });
                });
            });

        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(232.0)
            .frame(egui::Frame::none().fill(theme::PANEL).inner_margin(egui::Margin::same(12.0)))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                for tab in Tab::ALL {
                    let picked = self.tab == tab;
                    let text = egui::RichText::new(format!("{}   {}", tab.glyph(), tab.name()))
                        .size(14.5)
                        .color(if picked { theme::AMBER } else { theme::TEXT });
                    if ui
                        .add_sized(
                            [208.0, 34.0],
                            egui::Button::new(text)
                                .fill(if picked { theme::PANEL_HI } else { theme::PANEL })
                                .stroke(egui::Stroke::new(1.0, if picked { theme::AMBER } else { theme::LINE })),
                        )
                        .clicked()
                    {
                        self.tab = tab;
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(theme::label("GAME FOLDERS"));
                let homes = self.layout.homes.clone();
                if homes.is_empty() {
                    ui.label(theme::note("none found", false));
                }
                for (index, home) in homes.iter().enumerate() {
                    let picked = index == self.home_index;
                    let label = home
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| home.display().to_string());
                    if ui
                        .selectable_label(picked, egui::RichText::new(label).size(13.0))
                        .on_hover_text(home.display().to_string())
                        .clicked()
                        && !picked
                    {
                        self.home_index = index;
                        self.reload_worlds();
                    }
                }
                if let Some(home) = self.home() {
                    ui.add_space(6.0);
                    ui.label(theme::label(home.display().to_string()));
                }
            });

        egui::TopBottomPanel::bottom("log")
            .resizable(true)
            .default_height(130.0)
            .frame(egui::Frame::none().fill(theme::PANEL).inner_margin(egui::Margin::same(12.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(theme::label("WHAT THE TOOL DID"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Clear").clicked() {
                            self.log.clear();
                        }
                    });
                });
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for line in &self.log {
                            let color = if line.contains("✗") {
                                theme::RED
                            } else {
                                theme::DIM
                            };
                            ui.label(egui::RichText::new(line).monospace().size(12.0).color(color));
                        }
                    });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(theme::BG).inner_margin(egui::Margin::same(18.0)))
            .show(ctx, |ui| match self.tab {
                Tab::Profile => self.profile_page(ui),
                Tab::Player => self.player_page(ui),
                Tab::Inventory => self.inventory_page(ui),
                Tab::Backups => self.backups_page(ui),
            });

        if self.load_requested {
            self.load_requested = false;
            self.load_world();
        }
    }
}

impl App {
    fn profile_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title("Profile"));
        ui.label(
            theme::label("The name the game plays under when no account is signed in, kept in the game folder's horizon-player.txt."),
        );
        ui.add_space(10.0);
        let home = self.home();
        match &home {
            Some(home) => {
                let file = launcher::player_name_file(home);
                let current = launcher::offline_name(home).unwrap_or_else(|| "—".into());
                ui.horizontal(|ui| {
                    ui.label(theme::label("FILE"));
                    ui.label(theme::value(file.display().to_string()));
                });
                ui.horizontal(|ui| {
                    ui.label(theme::label("NAME NOW"));
                    ui.label(theme::value(current));
                });
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(theme::label("OFFLINE NAME"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.name_input)
                            .hint_text("Steve")
                            .desired_width(180.0),
                    );
                    if ui.button(theme::action("Set name")).clicked() {
                        self.set_name();
                    }
                });
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(theme::label("WHAT A PATCH TOUCHES"));
                ui.label(
                    egui::RichText::new(
                        "horizon-player.txt — the offline player name.\n\
                         level.dat — the player's health, hunger, experience and inventory.\n\n\
                         Every file is copied to the tool's backup folder before it is written, and \
                         the Backups page puts any copy back. Nothing else in the save changes.",
                    )
                    .size(13.0)
                    .color(theme::TEXT),
                );
                ui.add_space(10.0);
                ui.label(
                    theme::label(
                        "The launcher's own account check is not bypassed here: this tool patches the \
                         profile, not Horizon's sign-in.",
                    ),
                );
            }
            None => {
                ui.label(theme::note("No game folder: set HORIZON_HOME, or start the game once.", false));
            }
        }
    }

    fn player_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title("Player"));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(theme::label("WORLD"));
            let selected = self
                .worlds
                .get(self.world_index)
                .map(|w| w.name.clone())
                .unwrap_or_else(|| "no world".into());
            egui::ComboBox::from_id_salt("world")
                .selected_text(selected)
                .width(280.0)
                .show_ui(ui, |ui| {
                    let names: Vec<String> = self.worlds.iter().map(|w| w.name.clone()).collect();
                    for (index, name) in names.iter().enumerate() {
                        if ui
                            .selectable_label(self.world_index == index, name)
                            .clicked()
                            && self.world_index != index
                        {
                            self.world_index = index;
                            self.load_requested = true;
                        }
                    }
                });
            if ui.button("Reload").clicked() {
                self.load_world();
            }
        });
        ui.add_space(12.0);

        let Some(stats) = self.stats.clone() else {
            ui.label(theme::note("No player in this save yet: open the world once and save.", false));
            return;
        };

        egui::Grid::new("stats")
            .num_columns(4)
            .spacing([28.0, 10.0])
            .show(ui, |ui| {
                ui.label(theme::label("HEALTH"));
                ui.label(theme::label("HUNGER"));
                ui.label(theme::label("SATURATION"));
                ui.label(theme::label("AIR"));
                ui.end_row();
                ui.label(theme::value(format!("{:.1} / 20", stats.health)));
                ui.label(theme::value(format!("{} / 20", stats.food)));
                ui.label(theme::value(format!("{:.1}", stats.saturation)));
                ui.label(theme::value(stats.air.to_string()));
                ui.end_row();
                ui.label(theme::label("LEVEL"));
                ui.label(theme::label("POINTS"));
                ui.label(theme::label("EXHAUSTION"));
                ui.label(theme::label("ABSORPTION"));
                ui.end_row();
                ui.label(theme::value(stats.xp_level.to_string()));
                ui.label(theme::value(stats.xp_total.to_string()));
                ui.label(theme::value(format!("{:.2}", stats.exhaustion)));
                ui.label(theme::value(format!("{:.1}", stats.absorption)));
                ui.end_row();
            });

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized([150.0, 36.0], egui::Button::new(theme::action("♥  Heal")))
                .clicked()
            {
                self.heal();
            }
            if ui
                .add_sized([150.0, 36.0], egui::Button::new(theme::action("🍗  Feed")))
                .clicked()
            {
                self.feed();
            }
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(theme::label("LEVEL"));
            ui.add(egui::DragValue::new(&mut self.level_input).range(0..=30_000));
            if ui.button(theme::action("Set level")).clicked() {
                self.set_level();
            }
        });
    }

    fn inventory_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title("Inventory"));
        ui.add_space(6.0);
        let Some(stats) = self.stats.clone() else {
            ui.label(theme::note("No player in this save yet: open the world once and save.", false));
            return;
        };

        egui::ScrollArea::vertical()
            .max_height(330.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("items")
                    .num_columns(4)
                    .striped(true)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(theme::label("SLOT"));
                        ui.label(theme::label("ITEM"));
                        ui.label(theme::label("COUNT"));
                        ui.label("");
                        ui.end_row();
                        for item in &stats.items {
                            ui.label(theme::value(item.slot.to_string()));
                            ui.label(theme::value(item.id.clone()));
                            ui.label(theme::value(item.count.to_string()));
                            if ui.small_button("Remove").clicked() {
                                self.remove_item(item.slot);
                            }
                            ui.end_row();
                        }
                        if stats.items.is_empty() {
                            ui.label(theme::label("nothing in the inventory"));
                            ui.end_row();
                        }
                    });
            });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(theme::label("ADD"));
            ui.add(
                egui::TextEdit::singleline(&mut self.item_id)
                    .hint_text("minecraft:bread")
                    .desired_width(280.0),
            );
            egui::ComboBox::from_id_salt("suggest")
                .selected_text("▾")
                .show_ui(ui, |ui| {
                    for id in items::COMMON {
                        if ui.selectable_label(self.item_id == *id, *id).clicked() {
                            self.item_id = (*id).to_string();
                        }
                    }
                });
            ui.add(egui::DragValue::new(&mut self.item_count).range(1..=64));
            if ui.button(theme::action("Add item")).clicked() {
                self.add_item();
            }
        });
        ui.add_space(6.0);
        ui.label(theme::label(
            "Any id works, modded ones too (modid:item). Stacks go on the one there if it isn't full, else in the first free slot.",
        ));
    }

    fn backups_page(&mut self, ui: &mut egui::Ui) {
        ui.label(theme::title("Backups"));
        ui.label(theme::label(format!("{} kept in {}", self.store.entries.len(), self.store.root.display())));
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("Reload").clicked() {
                self.store.reload();
            }
        });
        ui.add_space(10.0);
        let entries = self.store.entries.clone();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("backups")
                    .num_columns(4)
                    .striped(true)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(theme::label("WHEN"));
                        ui.label(theme::label("WHAT"));
                        ui.label(theme::label("FILE"));
                        ui.label("");
                        ui.end_row();
                        for entry in &entries {
                            ui.label(theme::value(entry.when.to_string()));
                            ui.label(theme::value(entry.tool.clone()));
                            ui.label(
                                theme::value(
                                    entry
                                        .original
                                        .file_name()
                                        .map(|n| n.to_string_lossy().into_owned())
                                        .unwrap_or_default(),
                                ),
                            )
                            .on_hover_text(entry.original.display().to_string());
                            if ui.small_button("Restore").clicked() {
                                self.restore(entry.id.clone());
                            }
                            ui.end_row();
                        }
                        if entries.is_empty() {
                            ui.label(theme::label("nothing changed yet"));
                            ui.end_row();
                        }
                    });
            });
    }
}
