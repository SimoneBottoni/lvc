# Linear-map Vector Commitments

This repository implements the Lagrange polynomial variant of the linear-map vector commitment scheme presented in:

> **Linear‑map Vector Commitments and their Practical Applications**  
> *Matteo Campanelli, Anca Nitulescu (Protocol Labs), Carla Ràfols, Alexandros Zacharakis, Arantxa Zapico (Universitat
Pompeu Fabra)*  
> [IACR ePrint 2022/705](https://eprint.iacr.org/2022/705)

Vector commitments allow you to commit to a vector and later prove statements about linear combinations of its elements
without revealing the underlying data. The Lagrange approach uses polynomial interpolation to enable efficient
commitment generation and verification.

---

## Repository Structure

```markdown
root/
│
├── benches/
│ └── lvc.rs # criterion benchmarks for lvc operations
│
├── src/ # core implementation
│ ├── setup.rs # trusted setup
│ ├── commit.rs # commitment logic
│ ├── lvc.rs # lvc operations (open, verify)
│ └── util.rs # serialization helpers (serde support for Arkworks)
│
├── Cargo.toml
│
└── README.md
```

---

## Benchmarks

Run benchmarks:

```bash
cargo bench --bench lvc
```

---

## Acknowledgements

The implementation is based on the [Arkworks Algebra library](https://github.com/arkworks-rs/algebra).

---

## Disclaimer

This is a research-grade implementation, meant for experimentation and educational use.
Not suitable for production deployment.
