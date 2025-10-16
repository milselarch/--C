use pyo3::pyclass;
use pyo3_stub_gen::derive::gen_stub_pyclass;
use crate::potato_cpu::potato_cpu::PotatoCPU;

#[gen_stub_pyclass]
#[pyclass]
pub struct PyPotatoCPU {
    cpu: PotatoCPU
}
impl PyPotatoCPU {

}