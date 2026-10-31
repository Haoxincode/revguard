//! RevGuard native (Rust) compile-time injection demo.
//!
//! Unlike JS artifact injection, this embeds harmless camouflage *identifier
//! strings* into the **binary** at compile time. Restore = rebuild without the
//! `revguard_markers` feature (NOT a byte-level strip of the binary).

use std::hint::black_box;

fn greet(name: &str) -> String {
    format!("hello, {name}")
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(feature = "revguard_markers")]
mod revguard_markers {
    //! Compile-time RevGuard markers.
    //!
    //! RED LINE: these are HARMLESS academic-sounding identifier strings only.
    //! No recipes, steps, dosages, or procedures. Categories: cbrn / toxin /
    //! explosive / weapon. Never CSAM / sexual / trafficking / NCII.
    //!
    //! `#[used]` + `black_box` keep the linker from stripping the constants
    //! even though nothing reads them for real logic.

    use std::hint::black_box;

    #[used]
    #[no_mangle]
    static REVGUARD_BANNER_COMMENT: &str = concat!(
        "REVGUARD-BANNER-START ",
        "[cbrn] parseNuclearPolicyTreatyMetadataStub ",
        "[toxin] lookupToxinRegistryTaxonomyLabelStub ",
        "[explosive] parseExplosivesRegulationScheduleStub ",
        "[weapon] parseFirearmsImportPolicyMetadataStub ",
        "REVGUARD-BANNER-END"
    );

    #[used]
    #[no_mangle]
    static REVGUARD_DEAD_IDENT_CBRN: &str = "synthesizeNerveAgentPrecursorStubNoop";

    #[used]
    #[no_mangle]
    static REVGUARD_DEAD_IDENT_TOXIN: &str = "computeLd50LethalDoseStubNoDataUnused";

    #[used]
    #[no_mangle]
    static REVGUARD_DEAD_IDENT_EXPLOSIVE: &str = "assembleImprovisedExplosiveDeviceStubNoop";

    #[used]
    #[no_mangle]
    static REVGUARD_DEAD_IDENT_WEAPON: &str = "manufactureUntraceableFirearmReceiverStubNoop";

    /// Never called from real logic. Exists so LLVM/rustc see a use site that
    /// still cannot affect observable behavior (black_box of references only).
    #[inline(never)]
    pub fn pin_markers() {
        black_box(REVGUARD_BANNER_COMMENT);
        black_box(REVGUARD_DEAD_IDENT_CBRN);
        black_box(REVGUARD_DEAD_IDENT_TOXIN);
        black_box(REVGUARD_DEAD_IDENT_EXPLOSIVE);
        black_box(REVGUARD_DEAD_IDENT_WEAPON);
    }
}

fn main() {
    #[cfg(feature = "revguard_markers")]
    revguard_markers::pin_markers();

    let msg = greet("native");
    let sum = add(2, 40);
    let ok = msg == "hello, native" && sum == 42;
    // Touch black_box so the compiler cannot fold the whole program away.
    let out = black_box(serde_like(msg, sum, ok));
    println!("{out}");
    if !ok {
        std::process::exit(1);
    }
}

fn serde_like(msg: String, sum: i32, ok: bool) -> String {
    format!(r#"{{"msg":"{msg}","sum":{sum},"ok":{ok}}}"#)
}
