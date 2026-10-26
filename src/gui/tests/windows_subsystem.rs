// Launching the GUI from Explorer must not open a console window, so the
// executable has to be linked for the Windows GUI subsystem in every build
// profile, including the debug profile this test runs against.
#![cfg(windows)]

const IMAGE_SUBSYSTEM_WINDOWS_GUI: u16 = 2;

fn pe_subsystem(path: &str) -> u16 {
    let bytes = std::fs::read(path).expect("failed to read executable");
    let pe_offset = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
    assert_eq!(&bytes[pe_offset..pe_offset + 4], b"PE\0\0", "not a PE file");
    // PE signature (4) + COFF file header (20), then Subsystem sits at
    // offset 68 of the optional header for both PE32 and PE32+.
    let subsystem_offset = pe_offset + 4 + 20 + 68;
    u16::from_le_bytes(
        bytes[subsystem_offset..subsystem_offset + 2]
            .try_into()
            .unwrap(),
    )
}

#[test]
fn gui_executable_uses_windows_gui_subsystem() {
    let exe = env!("CARGO_BIN_EXE_httprunner-gui");
    assert_eq!(pe_subsystem(exe), IMAGE_SUBSYSTEM_WINDOWS_GUI);
}
