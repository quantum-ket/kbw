use crate::bitwise::*;
use crate::quantum_execution::QuantumExecution;
use num::{complex::Complex64, Zero};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rayon::prelude::*;
use std::f64::consts::FRAC_1_SQRT_2;

pub struct Dense {
    num_states: usize,
    state_0: Vec<Complex64>,
    state_1: Vec<Complex64>,
    state: bool,
}

impl Dense {
    pub fn new() -> Dense {
        Dense {
            num_states: 0,
            state_0: Vec::new(),
            state_1: Vec::new(),
            state: true,
        }
    }

    fn get_states(&mut self) -> (&mut [Complex64], &mut [Complex64]) {
        self.state = !self.state;
        if self.state {
            (&mut self.state_1, &mut self.state_0)
        } else {
            (&mut self.state_0, &mut self.state_1)
        }
    }

    fn get_current_state(&mut self) -> &mut [Complex64] {
        if self.state {
            &mut self.state_0
        } else {
            &mut self.state_1
        }
    }

    fn swap(&mut self, a: u32, b: u32) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                *amp = current_state[if is_one_at(state, a) != is_one_at(state, b) {
                    bit_flip(bit_flip(state, a), b)
                } else {
                    state
                }];
            });
    }

    fn pown(&mut self, qubits_size: usize, args: &str) {
        let (current_state, next_state) = self.get_states();
        let args: Vec<&str> = args.split(' ').collect();
        let x: u64 = args[0].parse().unwrap();
        let n: u64 = args[1].parse().unwrap();
        let l = bit_len(n);

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                let a_b = (state & ((1 << qubits_size) - 1)) as u64;
                let a = a_b >> l;
                let mut b = a_b & ((1 << l) - 1);
                b *= crate::bitwise::pown(x, a, n);
                let a_b = (a << l) | b;
                *amp = current_state[a_b as usize];
            });
    }
}

impl QuantumExecution for Dense {
    fn prepare_for_execution(&mut self, metrics: &ket::Metrics) -> Result<(), String> {
        if metrics.max_num_qubit > 32 {
            return Err(String::from(
                "Dense simulator do not allow more then 32 qubits.",
            ));
        }

        for plugin in metrics.plugins.iter() {
            if plugin != "pown" {
                return Err(format!("Plugin {} not available", plugin));
            }
        }

        self.num_states = 1 << metrics.max_num_qubit;

        self.state_0.resize(self.num_states, Complex64::zero());
        self.state_1.resize(self.num_states, Complex64::zero());

        self.state_0[0] = Complex64::new(1.0, 0.0);
        self.state = true;

        Ok(())
    }

    fn pauli_x(&mut self, target: u32, control: &[u32]) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                *amp = current_state[if ctrl_check(state, control) {
                    bit_flip(state, target)
                } else {
                    state
                }];
            });
    }

    fn pauli_y(&mut self, target: u32, control: &[u32]) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[bit_flip(state, target)]
                        * if is_one_at(state, target) {
                            Complex64::i()
                        } else {
                            -Complex64::i()
                        };
                } else {
                    *amp = current_state[state];
                }
            });
    }

    fn pauli_z(&mut self, target: u32, control: &[u32]) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) && is_one_at(state, target) {
                    *amp = -current_state[state];
                } else {
                    *amp = current_state[state];
                }
            });
    }

    fn hadamard(&mut self, target: u32, control: &[u32]) {
        let (current_state, next_state) = self.get_states();

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[bit_flip(state, target)] * FRAC_1_SQRT_2;
                } else {
                    *amp = Complex64::zero();
                }
            });

        current_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = *amp
                        * if is_one_at(state, target) {
                            -FRAC_1_SQRT_2
                        } else {
                            FRAC_1_SQRT_2
                        };
                }
            });

        next_state
            .par_iter_mut()
            .zip(current_state.par_iter())
            .for_each(|(next_amp, current_amp)| {
                *next_amp += *current_amp;
            });
    }

    fn phase(&mut self, lambda: f64, target: u32, control: &[u32]) {
        let (current_state, next_state) = self.get_states();

        let phase = Complex64::exp(lambda * Complex64::i());

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) && is_one_at(state, target) {
                    *amp = current_state[state] * phase;
                } else {
                    *amp = current_state[state];
                }
            });
    }

    fn rx(&mut self, theta: f64, target: u32, control: &[u32]) {
        let (current_state, next_state) = self.get_states();

        let cons_theta_2 = Complex64::from(f64::cos(theta / 2.0));
        let sin_theta_2 = -Complex64::i() * f64::sin(theta / 2.0);

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[bit_flip(state, target)] * sin_theta_2;
                } else {
                    *amp = Complex64::zero();
                }
            });

        current_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = *amp * cons_theta_2;
                }
            });

        next_state
            .par_iter_mut()
            .zip(current_state.par_iter())
            .for_each(|(next_amp, current_amp)| {
                *next_amp += *current_amp;
            });
    }

    fn ry(&mut self, theta: f64, target: u32, control: &[u32]) {
        let (current_state, next_state) = self.get_states();

        let cons_theta_2 = Complex64::from(f64::cos(theta / 2.0));
        let p_sin_theta_2 = Complex64::from(f64::sin(theta / 2.0));
        let m_sin_theta_2 = -p_sin_theta_2;

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[bit_flip(state, target)]
                        * if is_one_at(state, target) {
                            p_sin_theta_2
                        } else {
                            m_sin_theta_2
                        };
                } else {
                    *amp = Complex64::zero();
                }
            });

        current_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = *amp * cons_theta_2;
                }
            });

        next_state
            .par_iter_mut()
            .zip(current_state.par_iter())
            .for_each(|(next_amp, current_amp)| {
                *next_amp += *current_amp;
            });
    }

    fn rz(&mut self, theta: f64, target: u32, control: &[u32]) {
        let (current_state, next_state) = self.get_states();

        let phase_0 = Complex64::exp(-theta / 2.0 * Complex64::i());
        let phase_1 = Complex64::exp(theta / 2.0 * Complex64::i());

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                if ctrl_check(state, control) {
                    *amp = current_state[state]
                        * if is_one_at(state, target) {
                            phase_1
                        } else {
                            phase_0
                        };
                } else {
                    *amp = current_state[state];
                }
            });
    }

    fn measure(&mut self, target: u32) -> bool {
        let (current_state, next_state) = self.get_states();

        let p1: f64 = current_state
            .par_iter()
            .enumerate()
            .map(|(state, amp)| {
                if is_one_at(state, target) {
                    amp.l1_norm().powi(2)
                } else {
                    0.0
                }
            })
            .sum();

        let p0 = match 1.0 - p1 {
            p0 if p0 >= 0.0 => p0,
            _ => 0.0,
        };

        let result = WeightedIndex::new([p0, p1])
            .unwrap()
            .sample(&mut thread_rng())
            == 1;

        let p = 1.0 / f64::sqrt(if result { p1 } else { p0 });

        next_state
            .par_iter_mut()
            .enumerate()
            .for_each(|(state, amp)| {
                *amp = if is_one_at(state, target) == result {
                    current_state[state] * p
                } else {
                    Complex64::zero()
                };
            });

        result
    }

    fn dump(&mut self, qubits: &[u32]) -> ket::DumpData {
        let mut basis_states = Vec::new();
        let mut amplitudes_real = Vec::new();
        let mut amplitudes_img = Vec::new();

        let state = self.get_current_state();

        state
            .iter()
            .enumerate()
            .filter(|(_state, amp)| amp.l1_norm() > 1e-15)
            .for_each(|(state, amp)| {
                let state = qubits
                    .iter()
                    .rev()
                    .enumerate()
                    .map(|(index, qubit)| (is_one_at(state, *qubit) as usize) << index)
                    .reduce(|a, b| a | b)
                    .unwrap_or(0);

                basis_states.push(Vec::from([state as u64]));
                amplitudes_real.push(amp.re);
                amplitudes_img.push(amp.im);
            });

        ket::DumpData {
            basis_states,
            amplitudes_real,
            amplitudes_img,
        }
    }

    fn plugin(
        &mut self,
        _name: &str,
        target: &[u32],
        control: &[u32],
        adj: bool,
        args: &str,
    ) -> Result<(), String> {
        if adj {
            return Err(String::from("Plugin pown do not implement its inverse."));
        }
        if control.len() != 0 {
            return Err(String::from("Plugin pown do not accept control qubits."));
        }

        let mut pos: Vec<usize> = (0..target.len()).collect();
        let mut swap_list = Vec::new();

        for (index, qubit) in target.iter().enumerate() {
            if (*qubit as usize) == pos[index] {
                continue;
            };
            swap_list.push((index, pos[index]));
            let tmp = pos[index];
            pos[index] = pos[target[index] as usize];
            pos[target[index] as usize] = tmp;
        }

        for (a, b) in swap_list.iter() {
            self.swap(*a as u32, *b as u32);
        }

        self.pown(target.len(), args);

        for (a, b) in swap_list {
            self.swap(a as u32, b as u32);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use ket::*;

    fn bell() -> (Process, Qubit, Qubit) {
        let mut p = Process::new(0);
        let a = p.alloc(false).unwrap();
        let b = p.alloc(false).unwrap();
        p.apply_gate(QuantumGate::Hadamard, &a).unwrap();
        p.ctrl_push(&[&a]).unwrap();
        p.apply_gate(QuantumGate::PauliX, &b).unwrap();
        p.ctrl_pop().unwrap();
        (p, a, b)
    }

    #[test]
    fn dump_bell() {
        let (mut p, a, b) = bell();
        let d = p.dump(&[&a, &b]).unwrap();
        crate::run_dense_from_process(&mut p).unwrap();
        assert!(d.value().is_some());
        println!("{:?}", d);
    }

    #[test]
    fn measure_bell() {
        for _ in 0..10 {
            let (mut p, mut a, mut b) = bell();
            let m = p.measure(&mut [&mut a, &mut b]).unwrap();
            crate::run_dense_from_process(&mut p).unwrap();
            let m = m.value().unwrap();
            assert!(m == 0 || m == 3);
        }
    }

    #[test]
    fn dump_h_3() {
        let mut p = Process::new(0);
        let q: Vec<Qubit> = (0..3)
            .into_iter()
            .map(|_| p.alloc(false).unwrap())
            .collect();

        q.iter()
            .for_each(|q| p.apply_gate(QuantumGate::Hadamard, q).unwrap());

        let q: Vec<&Qubit> = q.iter().collect();
        let d = p.dump(&q).unwrap();
        crate::run_dense_from_process(&mut p).unwrap();
        assert!(d.value().is_some());
        println!("{:?}", d);
    }

    #[test]
    fn measure_hzh_20() {
        let mut p = Process::new(0);
        let mut q: Vec<Qubit> = (0..20)
            .into_iter()
            .map(|_| p.alloc(false).unwrap())
            .collect();

        q.iter()
            .for_each(|q| p.apply_gate(QuantumGate::Hadamard, q).unwrap());

        q.iter()
            .for_each(|q| p.apply_gate(QuantumGate::PauliZ, q).unwrap());

        q.iter()
            .for_each(|q| p.apply_gate(QuantumGate::Hadamard, q).unwrap());

        let mut q: Vec<&mut Qubit> = q.iter_mut().collect();
        let m = p.measure(&mut q).unwrap();
        crate::run_dense_from_process(&mut p).unwrap();
        assert!(m.value().unwrap() == ((1 << 20) - 1));
        println!("{:?}; Execution Time = {}", m, p.exec_time().unwrap());
    }

    #[test]
    fn measure_h_20() {
        let mut p = Process::new(0);
        let mut q: Vec<Qubit> = (0..20)
            .into_iter()
            .map(|_| p.alloc(false).unwrap())
            .collect();

        q.iter()
            .for_each(|q| p.apply_gate(QuantumGate::Hadamard, q).unwrap());

        let mut q: Vec<&mut Qubit> = q.iter_mut().collect();
        let m = p.measure(&mut q).unwrap();
        crate::run_dense_from_process(&mut p).unwrap();
        println!("{:?}; Execution Time = {}", m, p.exec_time().unwrap());
    }
}
