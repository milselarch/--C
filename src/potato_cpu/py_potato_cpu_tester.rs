use pyo3::{pyclass, pymethods, Bound, PyResult};
use pyo3::exceptions::PyValueError;
use pyo3::types::PyType;
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use crate::potato_cpu::potato_asm::PotatoProgram;
use crate::tacky;

#[gen_stub_pyclass]
#[pyclass]
pub struct PyPotatoCPUTester {
    program: PotatoProgram
}
#[gen_stub_pymethods]
#[pymethods]
impl PyPotatoCPUTester {
    #[classmethod]
    pub fn compile_from_source(
        _cls: &Bound<'_, PyType>, source_filepath: String
    ) -> PyResult<Self> {
        let tacky_gen_result =
            tacky::tacky_symbols::tacky_gen_from_filepath(&*source_filepath, true);
        let tacky_program = match tacky_gen_result {
            Ok(program) => { program }
            Err(_) => {
                return Err(PyValueError::new_err(format!(
                    "Tacky Generation Error: {}", tacky_gen_result.err().unwrap()
                )));
            }
        };
        let potato_program = PotatoProgram::from_tacky_program(tacky_program);
        Ok(Self { program: potato_program })
    }

    pub fn execute(&self) -> PyResult<i64> {
        let result = self.program.execute();
        Ok(result)
    }
}

define_stub_info_gatherer!(stub_info);
