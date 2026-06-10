# Mycelium Language — Example Programs & Usage Reference

**Version:** 0.2 (Extended)  
**Date:** 2026-06-10

This document demonstrates the lexicon in realistic code. Examples progress from common patterns to more advanced and niche use cases, grounded in the Core IR semantics defined in RFC-0001.

---

### Common Use Cases

#### 1. Defining a Colony with Types, Embody, and Grow

```mycelium
colony linear_algebra {

    type Vector {
        data: Dense{dim: 1024, dtype: F32},
    }

    embody Vector {
        fn dot(self, other: Vector) -> f64 { ... }
    }

    grow Debug for Vector;
    grow Serialize for Vector;
}
```

#### 2. Explicit Representation Swap with Guarantees

```mycelium
let t: Value<Ternary{trits: 16}> = ...;

let b: Value<Binary{width: 16}> = 
    swap(t, to: Binary{width: 16}, policy: lossless_within_range);

// Resulting value carries:
// b.meta.guarantee == Exact
// b.meta.bound == None
// b.meta.policy_used == Some(lossless_within_range)
```

#### 3. Spawning and Managing Hyphae

```mycelium
let worker: hyph = spawn_hyph {
    let data = load_data();
    process(data)
};

let result = worker.join();
```

#### 4. Anastomosis for Collaboration or Redundancy

```mycelium
let h1: hyph = spawn_hyph { compute_part_a() };
let h2: hyph = spawn_hyph { compute_part_b() };

anas(h1, h2); // fuse for shared state or load balancing
```

#### 5. Translocation of Data/Resources

```mycelium
let data: Value<Dense{dim: 4096, dtype: F16>> = ...;
xloc(data, to: target_hyph); // efficient movement across the network
```

#### 6. Sclerotization for Checkpointing & Resumption

```mycelium
let long_running: hyph = spawn_hyph { heavy_computation() };

let checkpoint: Sclerotium = sclrt(long_running);

// Later (same node or migrated)
let resumed = germinate(checkpoint);
```

#### 7. Mycorrhizal Symbiosis Declaration

```mycelium
myco(with: ComputeInfrastructure) {
    provides: [Compute, Memory],
    requires: [Power, Cooling],
    contract: mutual_benefit
}
```

#### 8. Using `grow` and `matured` for Stable Components

```mycelium
matured fn inference_pipeline(input: Value<Dense<...>>) -> Value<Dense<...>> {
    // heavily optimized path
}

grow Serialize for inference_pipeline;
```

#### 9. Wild Block for Controlled Unsafe Operations

```mycelium
fn safe_wrapper(data: Bytes) -> Result<Processed, Error> {
    wild {
        // Only raw FFI or manual memory here
        foreign_accelerator_call(data)
    }
}
```

#### 10. Basic Forage Policy

```mycelium
let task = spawn_hyph {
    forage(policy: load_balanced)
};
```

---

### Niche / Advanced Use Cases

#### 11. Chained Representation Swaps with Degrading Guarantees

```mycelium
let vsa_vec: Value<VSA{model: MAP, dim: 10000, sparsity: Sparse}> = ...;

// First swap to dense (lossy)
let dense: Value<Dense{dim: 10000, dtype: F32>> = 
    swap(vsa_vec, to: Dense{dim: 10000, dtype: F32}, policy: approximate);

// Guarantee has degraded
// dense.meta.guarantee == Empirical
// dense.meta.bound == Some(CapacityBound { ... })
```

#### 12. Combining Spore + Sclerotium for Resilient Deployment

```mycelium
let model_spore = spore {
    model: trained_vsa_model,
    reconstruction: manifest
};

let durable_checkpoint = sclrt(germinate(model_spore));

// Can be dispersed, germinated, or resumed from sclerotium later
```

#### 13. Using `rhizo` for High-Bandwidth Paths

```mycelium
let high_priority_path = rhizo {
    bandwidth: high,
    latency: low
};

xloc(large_tensor, via: high_priority_path);
```

#### 14. `cmn` for Emergent Coordination

```mycelium
cmn.broadcast(TrainingSignal::Converged {
    colony: current_colony,
    accuracy: 0.97
});

// Other hyphae can react to signals without direct coupling
```

#### 15. `dimorph` for Mode Switching

```mycelium
dimorph {
    dense_mode: {
        // high precision, higher resource use
    },
    sparse_mode: {
        // lower precision, better for edge / mobile
    }
}
```

#### 16. `forage` with Explicit Policy + EXPLAIN

```mycelium
let placement = forage(
    policy: cost_aware,
    explain: true
);

// Returns both the decision and the rationale
```

#### 17. Resource Reclamation in a Long-Running System

```mycelium
loop {
    let stale = detect_stale_hyphae();
    reclaim(stale);
}
```

#### 18. Full Multi-Representation Pipeline (Advanced)

```mycelium
let input: Value<Binary{width: 32}> = ...;

let ternary = swap(input, to: Ternary{trits: 32}, policy: lossless);
let vsa     = swap(ternary, to: VSA{...}, policy: lossy_approximate);

let result = process_vsa(vsa);

let final_binary = swap(result, to: Binary{width: 32}, policy: roundtrip_safe);
```

#### 19. Using `anas` + `xloc` + `sclrt` Together

```mycelium
let h1 = spawn_hyph { ... };
let h2 = spawn_hyph { ... };

anas(h1, h2);
xloc(critical_state, to: h2);

let checkpoint = sclrt(h2); // fused + translocated state is now checkpointed
```

#### 20. `myco` + `forage` for Self-Optimizing Infrastructure Interaction

```mycelium
myco(with: ClusterScheduler) {
    // declare what this colony needs
}

let worker = spawn_hyph {
    forage(policy: mycorrhizal_aware)
};
```

---

*End of Example Programs Reference v0.2*