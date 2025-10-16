use pyo3_stub_gen::Result;
use py_ca_compiler::potato_cpu::py_potato_cpu_tester;

fn main() -> Result<()> {
    // `stub_info` is a function defined by `define_stub_info_gatherer!` macro.
    let potato_cpu_stub = py_potato_cpu_tester::stub_info()?;
    potato_cpu_stub.generate()?;
    Ok(())
}