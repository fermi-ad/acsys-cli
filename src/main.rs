use clap::{clap_app, crate_version, crate_name};

fn main() {
    let _matches = clap_app!(
        (crate_name!()) =>
            (version: crate_version!())
            (about: "Command line utility to access Fermilab's accelerator data")
            (@subcommand get =>
             (about: "Retrieves accelerator data")
             (aliases: &["read", "request"])
             (@subcommand history =>
              (about: "Makes the retrieval get historical data from a logger")
              (@arg START: -s --start <TIME> +required "sets the start time of the range")
              (@arg END: -e --end <TIME> !required "sets the end time of the range")
              (@arg DRF: +required "specifies the device and rate for acquisition")
             )
             (@subcommand live =>
              (about: "Retrieves live data from the accelerator")
              (@arg DRF: +required "specifies the device and rate for acquisition")
             )
            )
            (@subcommand put =>
             (about: "Updates the value of a device")
             (aliases: &["modify", "update"])
             (@arg DRF: +required "specifies the device to be modified")
             (@arg VALUE: +required "specifies the new value for the device")
            )
    )
    .get_matches();
}
