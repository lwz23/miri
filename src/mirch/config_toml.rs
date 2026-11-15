use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct KMiriConfigToml {
    #[serde(default = "config_page_table")]
    page_table: bool,
    /// The upper limit of physical memory for the kernel.
    total_mem_size: u64,
    /// The upper limit of a spawned non-main thread.
    #[serde(default = "config_stack_mem_size")]
    stack_mem_size: u64,
    /// Linker-script / `extern` symbols that should keep a fixed address.
    /// Unknown `extern fn` symbols are rejected unless listed here (KMiri#76).
    /// The value is a physical address.
    #[serde(default)]
    layout: BTreeMap<String, u64>,
    #[serde(default)]
    kalloc: Vec<KAlloc>,
    /// Symbols defined in user code that should take precedence over built-in shims.
    /// This is useful when you have your own implementation of standard library functions
    /// (like `strlen`, `memcpy`, etc.) and want Miri to use them instead of reporting
    /// a symbol clash error as in <https://github.com/KMiri-rs/KMiri/issues/171>.
    ///
    /// Example:
    /// ```toml
    /// [user_preferred_symbol]
    /// strlen = true
    /// memcpy = true
    /// ```
    #[serde(default)]
    user_preferred_symbol: BTreeMap<String, bool>,
    /// Configuration for KMiri's optional interrupt model.
    #[serde(default)]
    irq: IrqConfigToml,
}

/// Configuration for deterministic and randomized interrupt injection.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct IrqConfigToml {
    enabled: bool,
    tick_interval_basic_blocks: u64,
    inject_at_basic_blocks: Vec<u64>,
    random_injection_rate: f64,
    adversarial_vectors: Vec<u8>,
    adversarial_injection_rate: f64,
    critical_injection_rate: f64,
    random_seed: u64,
    handler_budget_basic_blocks: u64,
    max_nesting_depth: u64,
    trace: bool,
}

impl Default for IrqConfigToml {
    fn default() -> Self {
        Self {
            enabled: false,
            tick_interval_basic_blocks: 0,
            inject_at_basic_blocks: Vec::new(),
            random_injection_rate: 0.0,
            adversarial_vectors: Vec::new(),
            adversarial_injection_rate: 0.0,
            critical_injection_rate: 0.0,
            random_seed: 0,
            handler_budget_basic_blocks: 10_000,
            max_nesting_depth: 2,
            trace: false,
        }
    }
}

impl IrqConfigToml {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn tick_interval_basic_blocks(&self) -> u64 {
        self.tick_interval_basic_blocks
    }

    pub fn inject_at_basic_blocks(&self) -> &[u64] {
        &self.inject_at_basic_blocks
    }

    pub fn random_injection_rate(&self) -> f64 {
        self.random_injection_rate
    }

    /// Vectors that the adversarial device event source may raise.
    ///
    /// The list is intentionally independent of OSTD's IRQ-line allocator: a
    /// real device may raise a line after its driver has released it, and the
    /// native OSTD dispatcher safely handles an IRQ with no callback.
    pub fn adversarial_vectors(&self) -> &[u8] {
        &self.adversarial_vectors
    }

    pub fn adversarial_injection_rate(&self) -> f64 {
        self.adversarial_injection_rate
    }

    /// The adversarial source is active only when both a vector list and a
    /// non-zero probability are configured. When inactive, its RNG is never
    /// drawn and therefore cannot perturb existing deterministic traces.
    pub fn adversarial_enabled(&self) -> bool {
        !self.adversarial_vectors.is_empty() && self.adversarial_injection_rate > 0.0
    }

    pub fn critical_injection_rate(&self) -> f64 {
        self.critical_injection_rate
    }

    /// Critical-point injection is active only when both `adversarial_vectors`
    /// is non-empty and `critical_injection_rate > 0.0`. When inactive, its RNG
    /// is never drawn and cannot perturb existing traces.
    pub fn critical_enabled(&self) -> bool {
        !self.adversarial_vectors.is_empty() && self.critical_injection_rate > 0.0
    }

    pub fn random_seed(&self) -> u64 {
        self.random_seed
    }

    pub fn handler_budget_basic_blocks(&self) -> u64 {
        self.handler_budget_basic_blocks
    }

    pub fn max_nesting_depth(&self) -> u64 {
        self.max_nesting_depth
    }

    pub fn trace(&self) -> bool {
        self.trace
    }
}

impl KMiriConfigToml {
    pub fn new(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|err| format!("failed to read `{}`: {err}", path.display()))?;
        let config: Self = basic_toml::from_str(&text)
            .map_err(|err| format!("failed to parse `{}`: {err}", path.display()))?;
        let rate = config.irq.random_injection_rate;
        if !rate.is_finite() || !(0.0..=1.0).contains(&rate) {
            return Err(format!(
                "invalid `[irq].random_injection_rate` in `{}`: expected a value in 0.0..=1.0",
                path.display()
            ));
        }
        let adversarial_rate = config.irq.adversarial_injection_rate;
        if !adversarial_rate.is_finite() || !(0.0..=1.0).contains(&adversarial_rate) {
            return Err(format!(
                "invalid `[irq].adversarial_injection_rate` in `{}`: expected a value in 0.0..=1.0",
                path.display()
            ));
        }
        if let Some(vector) = config
            .irq
            .adversarial_vectors
            .iter()
            .copied()
            .find(|vector| !(32..=255).contains(vector))
        {
            return Err(format!(
                "invalid `[irq].adversarial_vectors` in `{}`: vector {vector} is outside the valid range 32..=255",
                path.display()
            ));
        }
        let critical_rate = config.irq.critical_injection_rate;
        if !critical_rate.is_finite() || !(0.0..=1.0).contains(&critical_rate) {
            return Err(format!(
                "invalid `[irq].critical_injection_rate` in `{}`: expected a value in 0.0..=1.0",
                path.display()
            ));
        }
        if critical_rate > 0.0 && config.irq.adversarial_vectors.is_empty() {
            return Err(format!(
                "invalid `[irq].critical_injection_rate` in `{}`: positive rate requires non-empty `[irq].adversarial_vectors`",
                path.display()
            ));
        }
        if config.irq.enabled && config.irq.handler_budget_basic_blocks == 0 {
            return Err(format!(
                "invalid `[irq].handler_budget_basic_blocks` in `{}`: expected a non-zero value when IRQ simulation is enabled",
                path.display()
            ));
        }
        if config.irq.enabled && config.irq.max_nesting_depth == 0 {
            return Err(format!(
                "invalid `[irq].max_nesting_depth` in `{}`: expected a non-zero value when IRQ simulation is enabled",
                path.display()
            ));
        }
        Ok(config)
    }

    pub fn total_mem_size(&self) -> u64 {
        self.total_mem_size
    }

    pub fn stack_mem_size(&self) -> u64 {
        self.stack_mem_size
    }

    pub fn page_table_enabled(&self) -> bool {
        self.page_table
    }

    pub fn symbol_addr(&self, symbol: &str) -> Option<u64> {
        self.layout.get(symbol).copied()
    }

    pub fn layout_symbols(&self) -> impl Iterator<Item = (&str, u64)> {
        self.layout.iter().map(|(name, addr)| (name.as_str(), *addr))
    }

    pub fn get_kalloc(&self, base_addr: usize) -> Option<&KAlloc> {
        self.kalloc.iter().find(|kalloc| kalloc.base_addr == base_addr)
    }

    pub fn prefers_user_implementation(&self, symbol_name: &str) -> bool {
        self.user_preferred_symbol.get(symbol_name).copied().unwrap_or(false)
    }

    pub fn irq(&self) -> &IrqConfigToml {
        &self.irq
    }
}

/// Enable page table by default.
/// FIXME: default to false if asterinas migrates to toml config.
fn config_page_table() -> bool {
    true
}

pub fn config_stack_mem_size() -> u64 {
    0x4000 // 16 KB
}

#[derive(Clone, Debug, Deserialize)]
pub struct KAlloc {
    pub name: String,
    pub base_addr: usize,
    pub size: usize,
    pub align: usize,
}

#[test]
fn layout() {
    let config: KMiriConfigToml = basic_toml::from_str(
        "
[layout]
_stext   = 0x80000000
_etext   = 0x8001b600
_sflash  = 0x80000000
_eflash  = 0x80200000
_sapps   = 0x80100000
_eapps   = 0x80200000
_ssram   = 0x80200000
_esram   = 0x80400000
_sappmem = 0x8021b680
_eappmem = 0x80400000
",
    )
    .unwrap();
    assert_eq!(config.layout["_stext"], 0x80000000);
    assert_eq!(config.layout["_sapps"], 0x80100000);
    assert_eq!(config.layout["_eappmem"], 0x80400000);
}

#[test]
fn irq_defaults_and_values() {
    let config: KMiriConfigToml = basic_toml::from_str("total_mem_size = 0x8000000\n").unwrap();
    assert!(!config.irq().enabled());
    assert_eq!(config.irq().handler_budget_basic_blocks(), 10_000);
    assert_eq!(config.irq().max_nesting_depth(), 2);
    assert!(config.irq().adversarial_vectors().is_empty());
    assert_eq!(config.irq().adversarial_injection_rate(), 0.0);
    assert!(!config.irq().adversarial_enabled());
    assert_eq!(config.irq().critical_injection_rate(), 0.0);
    assert!(!config.irq().critical_enabled());

    let config: KMiriConfigToml = basic_toml::from_str(
        r#"
total_mem_size = 0x8000000

[irq]
enabled = true
tick_interval_basic_blocks = 8
inject_at_basic_blocks = [12, 20]
random_injection_rate = 0.25
adversarial_vectors = [64, 65]
adversarial_injection_rate = 0.00005
critical_injection_rate = 0.1
random_seed = 7
handler_budget_basic_blocks = 64
max_nesting_depth = 5
trace = true
"#,
    )
    .unwrap();
    assert!(config.irq().enabled());
    assert_eq!(config.irq().tick_interval_basic_blocks(), 8);
    assert_eq!(config.irq().inject_at_basic_blocks(), [12, 20]);
    assert_eq!(config.irq().random_injection_rate(), 0.25);
    assert_eq!(config.irq().adversarial_vectors(), [64, 65]);
    assert_eq!(config.irq().adversarial_injection_rate(), 0.00005);
    assert!(config.irq().adversarial_enabled());
    assert_eq!(config.irq().critical_injection_rate(), 0.1);
    assert!(config.irq().critical_enabled());
    assert_eq!(config.irq().random_seed(), 7);
    assert_eq!(config.irq().handler_budget_basic_blocks(), 64);
    assert_eq!(config.irq().max_nesting_depth(), 5);
    assert!(config.irq().trace());
}

#[test]
fn irq_rejects_invalid_adversarial_config() {
    let cases = [
        ("total_mem_size = 0x8000000\n[irq]\nadversarial_vectors = [31]\n", "adversarial_vectors"),
        (
            "total_mem_size = 0x8000000\n[irq]\nadversarial_injection_rate = 1.1\n",
            "adversarial_injection_rate",
        ),
    ];
    for (index, (source, expected)) in cases.iter().enumerate() {
        let path = std::env::temp_dir()
            .join(format!("kmiri-irq-adversarial-config-{}-{index}.toml", std::process::id()));
        std::fs::write(&path, source).unwrap();
        let err = KMiriConfigToml::new(&path).unwrap_err();
        assert!(err.contains(expected), "unexpected error: {err}");
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn irq_rejects_invalid_critical_config() {
    let cases = [
        (
            "total_mem_size = 0x8000000\n[irq]\nadversarial_vectors = [64]\ncritical_injection_rate = 1.1\n",
            "critical_injection_rate",
        ),
        (
            "total_mem_size = 0x8000000\n[irq]\nadversarial_vectors = [64]\ncritical_injection_rate = -0.1\n",
            "critical_injection_rate",
        ),
        (
            "total_mem_size = 0x8000000\n[irq]\ncritical_injection_rate = 0.1\n",
            "adversarial_vectors",
        ),
    ];
    for (index, (source, expected)) in cases.iter().enumerate() {
        let path = std::env::temp_dir()
            .join(format!("kmiri-irq-critical-config-{}-{index}.toml", std::process::id()));
        std::fs::write(&path, source).unwrap();
        let err = KMiriConfigToml::new(&path).unwrap_err();
        assert!(err.contains(expected), "unexpected error: {err}");
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn irq_rejects_zero_max_nesting_depth() {
    let path = std::env::temp_dir().join(format!("kmiri-irq-config-{}.toml", std::process::id()));
    std::fs::write(
        &path,
        "total_mem_size = 0x8000000\n[irq]\nenabled = true\nmax_nesting_depth = 0\n",
    )
    .unwrap();
    let err = KMiriConfigToml::new(&path).unwrap_err();
    assert!(err.contains("max_nesting_depth"));
    let _ = std::fs::remove_file(path);
}
