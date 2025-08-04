/// Benchmark específico para medir performance da integração DLL
/// 
/// Este benchmark mede o overhead da DLL comparado com implementações nativas
/// e mock, fornecendo métricas precisas para análise de performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_decimal::Decimal;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

use toucan_execution::{
    client::{
        corretora_brasileira::{CorretoraExecution, CorretoraConfig},
        mock::MockExecution,
    },
    ExecutionClient,
    order::request::OrderRequestOpen,
};

/// Estrutura para comparar diferentes tipos de execução
#[derive(Debug)]
enum ExecutionType {
    Mock,
    CorretoraDll,
    RestApi,     // Para comparação futura
    WebSocket,   // Para comparação futura
}

/// Configuração do benchmark
struct BenchmarkConfig {
    /// Número de ordens para teste de throughput
    order_count: usize,
    /// Latência simulada da rede (para mock)
    network_latency_ms: u64,
    /// Timeout para operações
    timeout_ms: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            order_count: 100,
            network_latency_ms: 10, // 10ms típico para Brasil
            timeout_ms: 5000,       // 5s timeout
        }
    }
}

/// Métricas de performance coletadas
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    /// Latência média em microsegundos
    avg_latency_us: u64,
    /// Latência P50 em microsegundos
    p50_latency_us: u64,
    /// Latência P95 em microsegundos
    p95_latency_us: u64,
    /// Latência P99 em microsegundos
    p99_latency_us: u64,
    /// Throughput (operações por segundo)
    throughput_ops_sec: f64,
    /// Taxa de sucesso
    success_rate: f64,
    /// Overhead comparado ao mock (em %)
    overhead_percent: Option<f64>,
}

/// Executa benchmark de latência para diferentes tipos de execução
fn benchmark_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    
    let mut group = c.benchmark_group("execution_latency");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(50);
    
    // Benchmark Mock (baseline)
    group.bench_function("mock_execution", |b| {
        let mock_client = MockExecution::new_with_latency(
            Duration::from_millis(config.network_latency_ms)
        );
        
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();
            
            // Simula operação simples (get balance)
            let _result = mock_client.fetch_balances().await;
            
            black_box(start.elapsed())
        });
    });
    
    // Benchmark DLL (se disponível)
    if std::env::var("SKIP_DLL_TESTS").is_err() {
        group.bench_function("dll_execution", |b| {
            let dll_config = CorretoraConfig {
                demo_mode: true,
                timeout_ms: config.timeout_ms,
                ..Default::default()
            };
            let dll_client = CorretoraExecution::new_with_config(dll_config);
            
            b.to_async(&rt).iter(|| async {
                let start = Instant::now();
                
                // Simula operação simples (get balance)
                let _result = dll_client.fetch_balances().await;
                
                black_box(start.elapsed())
            });
        });
    }
    
    group.finish();
}

/// Executa benchmark de throughput para processar múltiplas ordens
fn benchmark_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    
    let mut group = c.benchmark_group("execution_throughput");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);
    
    for &order_count in &[10, 50, 100, 200] {
        group.bench_with_input(
            BenchmarkId::new("mock", order_count),
            &order_count,
            |b, &order_count| {
                let mock_client = MockExecution::new_with_latency(
                    Duration::from_millis(config.network_latency_ms)
                );
                
                b.to_async(&rt).iter(|| async {
                    let start = Instant::now();
                    
                    // Processa múltiplas ordens em paralelo
                    let tasks: Vec<_> = (0..order_count)
                        .map(|i| {
                            let client = mock_client.clone();
                            tokio::spawn(async move {
                                // Simula colocação de ordem
                                let _result = client.fetch_balances().await;
                                i
                            })
                        })
                        .collect();
                    
                    // Aguarda todas as tasks
                    for task in tasks {
                        let _ = task.await;
                    }
                    
                    let elapsed = start.elapsed();
                    let throughput = order_count as f64 / elapsed.as_secs_f64();
                    
                    black_box((elapsed, throughput))
                });
            },
        );
        
        // Benchmark DLL (se disponível)
        if std::env::var("SKIP_DLL_TESTS").is_err() {
            group.bench_with_input(
                BenchmarkId::new("dll", order_count),
                &order_count,
                |b, &order_count| {
                    let dll_config = CorretoraConfig {
                        demo_mode: true,
                        timeout_ms: config.timeout_ms,
                        ..Default::default()
                    };
                    let dll_client = CorretoraExecution::new_with_config(dll_config);
                    
                    b.to_async(&rt).iter(|| async {
                        let start = Instant::now();
                        
                        // Processa múltiplas ordens sequencialmente (DLL pode não suportar concorrência)
                        for i in 0..order_count {
                            let _result = dll_client.fetch_balances().await;
                            black_box(i);
                        }
                        
                        let elapsed = start.elapsed();
                        let throughput = order_count as f64 / elapsed.as_secs_f64();
                        
                        black_box((elapsed, throughput))
                    });
                },
            );
        }
    }
    
    group.finish();
}

/// Executa benchmark de overhead da DLL vs implementações nativas
fn benchmark_dll_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("dll_overhead");
    group.measurement_time(Duration::from_secs(30));
    
    // Baseline: operação nativa Rust (simulação)
    group.bench_function("native_rust", |b| {
        b.iter(|| {
            let start = Instant::now();
            
            // Simula operação nativa (processamento local)
            let data = black_box(vec![1u64; 1000]);
            let sum: u64 = data.iter().sum();
            
            black_box((start.elapsed(), sum))
        });
    });
    
    // Mock com latência mínima
    group.bench_function("mock_minimal", |b| {
        let mock_client = MockExecution::new_with_latency(Duration::from_millis(1));
        
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();
            let _result = mock_client.fetch_balances().await;
            black_box(start.elapsed())
        });
    });
    
    // DLL (se disponível)
    if std::env::var("SKIP_DLL_TESTS").is_err() {
        group.bench_function("dll_call", |b| {
            let dll_config = CorretoraConfig {
                demo_mode: true,
                timeout_ms: 1000,
                ..Default::default()
            };
            let dll_client = CorretoraExecution::new_with_config(dll_config);
            
            b.to_async(&rt).iter(|| async {
                let start = Instant::now();
                let _result = dll_client.fetch_balances().await;
                black_box(start.elapsed())
            });
        });
    }
    
    group.finish();
}

/// Executa teste de stress para verificar estabilidade da DLL
fn benchmark_stress_test(c: &mut Criterion) {
    if std::env::var("SKIP_DLL_TESTS").is_ok() {
        return;
    }
    
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("dll_stress");
    group.measurement_time(Duration::from_secs(120)); // 2 minutos
    group.sample_size(5);
    
    group.bench_function("dll_sustained_load", |b| {
        let dll_config = CorretoraConfig {
            demo_mode: true,
            timeout_ms: 10000,
            ..Default::default()
        };
        let dll_client = CorretoraExecution::new_with_config(dll_config);
        
        b.to_async(&rt).iter(|| async {
            let start = Instant::now();
            let mut success_count = 0;
            let mut error_count = 0;
            
            // Executa operações por 10 segundos
            while start.elapsed() < Duration::from_secs(10) {
                match dll_client.fetch_balances().await {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
                
                // Pequena pausa para evitar saturar a DLL
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            
            let total_ops = success_count + error_count;
            let success_rate = if total_ops > 0 {
                success_count as f64 / total_ops as f64
            } else {
                0.0
            };
            
            black_box((total_ops, success_rate, start.elapsed()))
        });
    });
    
    group.finish();
}

/// Função auxiliar para calcular percentis
fn calculate_percentiles(mut latencies: Vec<u64>) -> (u64, u64, u64) {
    latencies.sort_unstable();
    let len = latencies.len();
    
    if len == 0 {
        return (0, 0, 0);
    }
    
    let p50_idx = len * 50 / 100;
    let p95_idx = len * 95 / 100;
    let p99_idx = len * 99 / 100;
    
    (
        latencies[p50_idx.min(len - 1)],
        latencies[p95_idx.min(len - 1)],
        latencies[p99_idx.min(len - 1)],
    )
}

/// Função de análise e relatório de performance
pub fn generate_performance_report() -> PerformanceMetrics {
    println!("\n🔍 RELATÓRIO DE PERFORMANCE - DLL vs IMPLEMENTAÇÕES NATIVAS\n");
    
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    
    // Coleta métricas do Mock (baseline)
    let mock_metrics = rt.block_on(async {
        let mock_client = MockExecution::new_with_latency(
            Duration::from_millis(config.network_latency_ms)
        );
        
        let mut latencies = Vec::new();
        let start_time = Instant::now();
        let mut success_count = 0;
        
        for _ in 0..config.order_count {
            let op_start = Instant::now();
            match mock_client.fetch_balances().await {
                Ok(_) => {
                    success_count += 1;
                    latencies.push(op_start.elapsed().as_micros() as u64);
                }
                Err(_) => {}
            }
        }
        
        let total_time = start_time.elapsed();
        let (p50, p95, p99) = calculate_percentiles(latencies.clone());
        let avg = if !latencies.is_empty() {
            latencies.iter().sum::<u64>() / latencies.len() as u64
        } else {
            0
        };
        
        PerformanceMetrics {
            avg_latency_us: avg,
            p50_latency_us: p50,
            p95_latency_us: p95,
            p99_latency_us: p99,
            throughput_ops_sec: success_count as f64 / total_time.as_secs_f64(),
            success_rate: success_count as f64 / config.order_count as f64,
            overhead_percent: None,
        }
    });
    
    println!("📊 MOCK EXECUTION (Baseline):");
    println!("   Latência Média: {:>8} μs ({:.2} ms)", mock_metrics.avg_latency_us, mock_metrics.avg_latency_us as f64 / 1000.0);
    println!("   P50:           {:>8} μs ({:.2} ms)", mock_metrics.p50_latency_us, mock_metrics.p50_latency_us as f64 / 1000.0);
    println!("   P95:           {:>8} μs ({:.2} ms)", mock_metrics.p95_latency_us, mock_metrics.p95_latency_us as f64 / 1000.0);
    println!("   P99:           {:>8} μs ({:.2} ms)", mock_metrics.p99_latency_us, mock_metrics.p99_latency_us as f64 / 1000.0);
    println!("   Throughput:    {:>8.2} ops/sec", mock_metrics.throughput_ops_sec);
    println!("   Taxa Sucesso:  {:>8.1}%", mock_metrics.success_rate * 100.0);
    
    // Se DLL disponível, coleta métricas da DLL
    if std::env::var("SKIP_DLL_TESTS").is_err() {
        let dll_metrics = rt.block_on(async {
            let dll_config = CorretoraConfig {
                demo_mode: true,
                timeout_ms: config.timeout_ms,
                ..Default::default()
            };
            let dll_client = CorretoraExecution::new_with_config(dll_config);
            
            let mut latencies = Vec::new();
            let start_time = Instant::now();
            let mut success_count = 0;
            
            for _ in 0..config.order_count {
                let op_start = Instant::now();
                match dll_client.fetch_balances().await {
                    Ok(_) => {
                        success_count += 1;
                        latencies.push(op_start.elapsed().as_micros() as u64);
                    }
                    Err(_) => {}
                }
            }
            
            let total_time = start_time.elapsed();
            let (p50, p95, p99) = calculate_percentiles(latencies.clone());
            let avg = if !latencies.is_empty() {
                latencies.iter().sum::<u64>() / latencies.len() as u64
            } else {
                0
            };
            
            let overhead = if mock_metrics.avg_latency_us > 0 {
                Some(((avg as f64 - mock_metrics.avg_latency_us as f64) / mock_metrics.avg_latency_us as f64) * 100.0)
            } else {
                None
            };
            
            PerformanceMetrics {
                avg_latency_us: avg,
                p50_latency_us: p50,
                p95_latency_us: p95,
                p99_latency_us: p99,
                throughput_ops_sec: success_count as f64 / total_time.as_secs_f64(),
                success_rate: success_count as f64 / config.order_count as f64,
                overhead_percent: overhead,
            }
        });
        
        println!("\n🔧 DLL EXECUTION:");
        println!("   Latência Média: {:>8} μs ({:.2} ms)", dll_metrics.avg_latency_us, dll_metrics.avg_latency_us as f64 / 1000.0);
        println!("   P50:           {:>8} μs ({:.2} ms)", dll_metrics.p50_latency_us, dll_metrics.p50_latency_us as f64 / 1000.0);
        println!("   P95:           {:>8} μs ({:.2} ms)", dll_metrics.p95_latency_us, dll_metrics.p95_latency_us as f64 / 1000.0);
        println!("   P99:           {:>8} μs ({:.2} ms)", dll_metrics.p99_latency_us, dll_metrics.p99_latency_us as f64 / 1000.0);
        println!("   Throughput:    {:>8.2} ops/sec", dll_metrics.throughput_ops_sec);
        println!("   Taxa Sucesso:  {:>8.1}%", dll_metrics.success_rate * 100.0);
        
        if let Some(overhead) = dll_metrics.overhead_percent {
            println!("   Overhead:      {:>8.1}%", overhead);
        }
        
        // Análise e recomendações
        println!("\n📈 ANÁLISE:");
        
        let latency_diff = dll_metrics.avg_latency_us as i64 - mock_metrics.avg_latency_us as i64;
        if latency_diff > 5000 { // > 5ms
            println!("   ⚠️  DLL adiciona latência significativa (+{:.2}ms)", latency_diff as f64 / 1000.0);
        } else if latency_diff > 1000 { // > 1ms
            println!("   ⚡ DLL adiciona latência moderada (+{:.2}ms)", latency_diff as f64 / 1000.0);
        } else {
            println!("   ✅ DLL tem overhead mínimo (+{:.2}ms)", latency_diff as f64 / 1000.0);
        }
        
        let throughput_ratio = dll_metrics.throughput_ops_sec / mock_metrics.throughput_ops_sec;
        if throughput_ratio < 0.8 {
            println!("   📉 DLL reduz throughput significativamente ({:.1}% do mock)", throughput_ratio * 100.0);
        } else if throughput_ratio < 0.95 {
            println!("   📊 DLL reduz throughput moderadamente ({:.1}% do mock)", throughput_ratio * 100.0);
        } else {
            println!("   🚀 DLL mantém throughput similar ao mock ({:.1}%)", throughput_ratio * 100.0);
        }
        
        dll_metrics
    } else {
        println!("\n⚠️  DLL BENCHMARKS SKIPPED (set SKIP_DLL_TESTS=false to enable)");
        mock_metrics.clone()
    }
}

// Configuração dos benchmarks
criterion_group!(
    benches,
    benchmark_latency,
    benchmark_throughput,
    benchmark_dll_overhead,
    benchmark_stress_test
);

criterion_main!(benches);

// Para executar individualmente
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_report() {
        let _metrics = generate_performance_report();
        // Report é impresso no stdout
    }
    
    #[test]
    fn test_percentile_calculation() {
        let latencies = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];
        let (p50, p95, p99) = calculate_percentiles(latencies);
        
        assert_eq!(p50, 500);  // 50th percentile
        assert_eq!(p95, 950);  // 95th percentile  
        assert_eq!(p99, 990);  // 99th percentile
    }
}
