use cursive::Cursive;
use cursive::views::*;
use paperspace::{PsMsg, PsSender, Machines, MachineState};

pub type UiReceiver = crossbeam_channel::Receiver<UiMsg>;
pub type UiSender = crossbeam_channel::Sender<UiMsg>;

pub enum UiMsg {
    ReloadServerList(Machines),
    ShowRefreshError(String),
    ShowLoadingPopup,
    HideLoadingPopup,
}

pub struct Ui {
    app: Cursive,
    ui_rx: UiReceiver,
    ps_tx: PsSender,
}

impl Ui {
    pub fn new(ui_rx: UiReceiver, ps_tx: PsSender) -> Self {
        let mut app = Cursive::default();

        app.set_user_data(ps_tx.clone());

        app.add_layer(Dialog::around(
            IdView::new("machine_list", ListView::new()))
            .padding(cursive::view::Margins::new(4,4,1,1))
            .title("Paperspace machines")
            .button(
                "Refresh",
                |app| {
                    app.user_data::<PsSender>().unwrap().send(
                        PsMsg::ReloadServers
                    ).unwrap();
                }
            )
            .button("Exit", |app| app.quit())
        );


        app.set_autorefresh(true);

        Self {
            app,
            ui_rx,
            ps_tx,
        }
    }

    fn show_refresh_error(&mut self, error: String) {
        self.app.call_on_id("machine_list", |machine_list: &mut ListView| {
            machine_list.clear();
            machine_list.add_child(
            &format!("Error loading servers: {}", error),
                DummyView
            );
        });
    }

    fn reload_servers_list(&mut self, machines: Machines) {
        self.app.call_on_id("machine_list", |machine_list: &mut ListView| {
            machine_list.clear();
            for machine in machines {
                let mut machine_row = LinearLayout::horizontal();
                machine_row.add_child(TextView::new(format!(" - {} ",&machine.state)));
                match machine.state {
                    MachineState::Off => {
                        let start_id = machine.id.clone();
                        machine_row.add_child(Button::new("Start", move |app| {
                            app.user_data::<PsSender>().unwrap().send(
                                PsMsg::StartServer(start_id.clone())
                            ).unwrap();
                        }));
                    },
                    MachineState::Provisioning => {},
                    _ => {
                        let stop_id = machine.id.clone();
                        machine_row.add_child(Button::new("Stop", move |app| {
                            app.user_data::<PsSender>().unwrap().send(
                                PsMsg::StopServer(stop_id.clone())
                            ).unwrap();
                        }));
                        let restart_id = machine.id.clone();
                        machine_row.add_child(Button::new("Restart", move |app| {
                            app.user_data::<PsSender>().unwrap().send(
                                PsMsg::RestartServer(restart_id.clone())
                            ).unwrap();
                        }));
                    }
                }
                machine_list.add_child(&machine.name, machine_row);
            };
        });
    }

    fn show_loading_popup(&mut self) {
        if self.app.find_id::<TextView>("loading_popup").is_none() {
            let refresh_popup = Dialog::around(IdView::new(
                "loading_popup",
                TextView::new("Sending request.."))
            ).title("Please wait");
            self.app.add_layer(refresh_popup)
        }
    }

    fn hide_loading_popup(&mut self) {
        if self.app.find_id::<TextView>("loading_popup").is_some() {
            self.app.pop_layer();
        }
    }

    pub fn run(&mut self) {
        // Populate the servers list on start
        self.ps_tx.send(PsMsg::ReloadServers).unwrap();
        
        // main loop
        while self.app.is_running() {
            match self.ui_rx.try_recv() {
                Ok(message) => {
                    match message {
                        UiMsg::ShowRefreshError(error) => self.show_refresh_error(error),
                        UiMsg::ReloadServerList(machines) => self.reload_servers_list(machines),
                        UiMsg::ShowLoadingPopup => self.show_loading_popup(),
                        UiMsg::HideLoadingPopup => self.hide_loading_popup(),
                    }
                },
                Err(reason) => if reason.is_disconnected() {
                    // Paperspace thread gone, let's quit
                    self.app.quit() 
                }
            }
            self.app.step();
        }
    }
}