const LIBLINEAR_REPO: &str = "https://github.com/cjlin1/liblinear.git";
const LIBLINEAR_TAG: &str = "v245";

fn main() {
    let tempdir = tempfile::tempdir().unwrap();

    let repo = git2::Repository::clone(LIBLINEAR_REPO, tempdir.path()).unwrap();
    let (obj, reference) = repo.revparse_ext(LIBLINEAR_TAG).unwrap();

    repo.checkout_tree(&obj, None).unwrap();
    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(obj.id()),
    }
    .unwrap();

    let p = tempdir.path();

    let mut b = cc::Build::new();
    b.cpp(true).flag("-O2");

    for f in &[
        "linear.cpp",
        "newton.cpp",
        "blas/daxpy.c",
        "blas/ddot.c",
        "blas/dnrm2.c",
        "blas/dscal.c",
    ] {
        b.file(p.join(f));
    }

    b.include(&p).include(p.join("blas")).compile("liblinear");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(p.join("linear.h").to_str().unwrap())
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
