use clap::{clap_app, crate_name, crate_version};

mod drf;

// Returns a data type that handles all details of command line
// arguments.

fn cmd_cfg() -> clap::App<'static, 'static> {
    clap_app!(
        (crate_name!()) =>
            (version: crate_version!())
            (about: "Command line utility to access Fermilab's accelerator data")

            // The GET subcommand is used to retrieve accelerator
            // data. It has its own subcommands.

            (@subcommand get =>
             (about: "Retrieves accelerator data (also: READ, REQUEST)")
             (aliases: &["read", "request"])

             // HISTORY is a subcommand of GET in which the data for
             // the requested devices comes from a data logger.

             (@subcommand history =>
              (about: "Makes the retrieval get historical data from a logger")
              (@arg START: -s --start <TIME> +required "sets the start time of the range")
              (@arg END: -e --end <TIME> !required "sets the end time of the range")
              (@arg DRF: +required "specifies the device and rate for acquisition")
             )

             // LIVE is a subcommand of get which returns live
             // accelerator data.

             (@subcommand live =>
              (about: "Retrieves live data from the accelerator")
              (@arg DRF: +required "specifies the device and rate for acquisition")
             )
            )

            // The PUT subcommand allows settings to be sent to devices.

            (@subcommand put =>
             (about: "Updates the value of a device (also: MODIFY, UPDATE)")
             (aliases: &["modify", "update"])
             (@arg DRF: +required "specifies the device to be modified")
             (@arg VALUE: +required "specifies the new value for the device")
            )
    )
}

fn main() {
    let _matches = cmd_cfg().get_matches();
}
