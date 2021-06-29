use clap::clap_app;

fn main() {
    let _matches = clap_app!(
        myapp =>
            (version: "0.1.0")
            (about: "Command line utility to access Fermilab's accelerator data")
            (@subcommand get =>
             (about: "get device data")
             (@arg DRF: +required "specified the device and rate for acquisition")
            )
    ).get_matches();
}
