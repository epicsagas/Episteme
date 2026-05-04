#!/usr/bin/env python3
"""
Performance Benchmark Suite for Syntagma
Measures code smell detection, API latency, and throughput
"""

import time
import statistics
import json
import sys
from pathlib import Path
from typing import List, Dict, Any
from dataclasses import dataclass, asdict
import requests

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from scripts.language_parsers import get_parser
from scripts.detect_smells import CodeSmellDetector
from scripts.refactoring_inference import RefactoringInferenceEngine


@dataclass
class BenchmarkResult:
    """Benchmark result with statistics"""
    name: str
    iterations: int
    mean_ms: float
    median_ms: float
    p95_ms: float
    p99_ms: float
    min_ms: float
    max_ms: float
    std_dev_ms: float


class PerformanceBenchmark:
    """Performance benchmarking suite"""

    def __init__(self, iterations: int = 100):
        self.iterations = iterations
        self.results: List[BenchmarkResult] = []

    def measure(self, name: str, func, *args, **kwargs) -> BenchmarkResult:
        """Measure function performance over multiple iterations"""
        timings = []

        # Warmup
        for _ in range(min(10, self.iterations // 10)):
            func(*args, **kwargs)

        # Actual measurement
        for _ in range(self.iterations):
            start = time.perf_counter()
            func(*args, **kwargs)
            end = time.perf_counter()
            timings.append((end - start) * 1000)  # Convert to ms

        result = BenchmarkResult(
            name=name,
            iterations=self.iterations,
            mean_ms=statistics.mean(timings),
            median_ms=statistics.median(timings),
            p95_ms=self._percentile(timings, 95),
            p99_ms=self._percentile(timings, 99),
            min_ms=min(timings),
            max_ms=max(timings),
            std_dev_ms=statistics.stdev(timings) if len(timings) > 1 else 0
        )

        self.results.append(result)
        return result

    def _percentile(self, data: List[float], percentile: int) -> float:
        """Calculate percentile"""
        sorted_data = sorted(data)
        index = int(len(sorted_data) * percentile / 100)
        return sorted_data[min(index, len(sorted_data) - 1)]

    def print_results(self):
        """Print benchmark results in table format"""
        print("\n" + "=" * 100)
        print("SYNTAGMA PERFORMANCE BENCHMARK RESULTS")
        print("=" * 100)
        print(f"\n{'Benchmark':<40} {'Mean':<10} {'Median':<10} {'P95':<10} {'P99':<10} {'Min':<10} {'Max':<10}")
        print("-" * 100)

        for result in self.results:
            print(f"{result.name:<40} "
                  f"{result.mean_ms:>8.2f}ms "
                  f"{result.median_ms:>8.2f}ms "
                  f"{result.p95_ms:>8.2f}ms "
                  f"{result.p99_ms:>8.2f}ms "
                  f"{result.min_ms:>8.2f}ms "
                  f"{result.max_ms:>8.2f}ms")

        print("=" * 100)

    def export_json(self, filepath: str):
        """Export results to JSON"""
        data = {
            'benchmark_date': time.strftime('%Y-%m-%d %H:%M:%S'),
            'iterations': self.iterations,
            'results': [asdict(r) for r in self.results]
        }

        with open(filepath, 'w') as f:
            json.dump(data, f, indent=2)

        print(f"\n✅ Results exported to {filepath}")


# Sample code for benchmarking
SAMPLE_PYTHON_CODE = '''
def process_order(customer_id, product_id, quantity, discount_code,
                  shipping_address, billing_address, payment_method,
                  gift_wrap, special_instructions):
    """Long method with many parameters"""
    if customer_id is None:
        return None
    if customer_id < 0:
        return None
    if product_id is None:
        return None
    if quantity is None:
        return None
    if quantity < 1:
        return None

    base_price = 100.00
    total = base_price * quantity

    if discount_code == "SAVE10":
        total = total * 0.9
    elif discount_code == "SAVE20":
        total = total * 0.8
    elif discount_code == "SAVE30":
        total = total * 0.7
    elif discount_code == "SAVE40":
        total = total * 0.6

    return total
'''

SAMPLE_JAVA_CODE = '''
public class OrderProcessor {
    private String customerId;
    private String productId;
    private int quantity;
    private String discountCode;
    private String shippingAddress;
    private String billingAddress;

    public double processOrder(String customerId, String productId,
                               int quantity, String discountCode,
                               String shippingAddress, String billingAddress,
                               String paymentMethod, boolean giftWrap) {
        if (customerId == null) return 0.0;
        if (productId == null) return 0.0;
        if (quantity < 1) return 0.0;

        double basePrice = 100.0;
        double total = basePrice * quantity;

        if ("SAVE10".equals(discountCode)) {
            total = total * 0.9;
        } else if ("SAVE20".equals(discountCode)) {
            total = total * 0.8;
        } else if ("SAVE30".equals(discountCode)) {
            total = total * 0.7;
        }

        return total;
    }
}
'''


def benchmark_python_parser(bench: PerformanceBenchmark):
    """Benchmark Python parser"""
    parser = get_parser('python')
    bench.measure("Python Parser - Code Analysis", parser.parse_code, SAMPLE_PYTHON_CODE, "test.py")


def benchmark_java_parser(bench: PerformanceBenchmark):
    """Benchmark Java parser"""
    parser = get_parser('java')
    bench.measure("Java Parser - Code Analysis", parser.parse_code, SAMPLE_JAVA_CODE, "test.java")


def benchmark_multi_language(bench: PerformanceBenchmark):
    """Benchmark all language parsers"""
    languages = {
        'python': SAMPLE_PYTHON_CODE,
        'java': SAMPLE_JAVA_CODE,
        'typescript': 'function longMethod(a, b, c, d, e, f, g, h) { if (a) { if (b) { return c; } } }',
        'go': 'func longMethod(a, b, c, d, e, f string) int { if a != "" { if b != "" { return 1 } } return 0 }',
        'rust': 'fn long_method(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 { if a > 0 { if b > 0 { return c; } } 0 }'
    }

    for lang, code in languages.items():
        parser = get_parser(lang)
        bench.measure(f"{lang.capitalize()} Parser", parser.parse_code, code, f"test.{lang}")


def benchmark_api_endpoints(bench: PerformanceBenchmark, base_url: str = "http://localhost:8000"):
    """Benchmark API endpoints (requires running server)"""
    try:
        # Test health endpoint
        bench.measure("API - Health Check",
                     lambda: requests.get(f"{base_url}/health", timeout=5))

        # Test analyze endpoint
        analyze_payload = {"code": SAMPLE_PYTHON_CODE, "language": "python", "min_confidence": 0.5}
        bench.measure("API - Analyze Endpoint",
                     lambda: requests.post(f"{base_url}/analyze", json=analyze_payload, timeout=10))

        # Test search endpoint
        search_payload = {"query": "Long Method fix", "top_k": 5}
        bench.measure("API - Search Endpoint",
                     lambda: requests.post(f"{base_url}/search", json=search_payload, timeout=10))

        # Test graph endpoint
        bench.measure("API - Graph Entity",
                     lambda: requests.get(f"{base_url}/graph/SMELL-01", timeout=5))

        print("✅ API benchmarks completed")

    except requests.exceptions.ConnectionError:
        print("⚠️  API server not running - skipping API benchmarks")
        print("   Start server with: uvicorn api.main:app --reload")


def main():
    """Run all benchmarks"""
    print("🚀 Starting Syntagma Performance Benchmark Suite")
    print(f"Iterations: 100 per benchmark\n")

    bench = PerformanceBenchmark(iterations=100)

    # Parser benchmarks
    print("📊 Benchmarking language parsers...")
    benchmark_python_parser(bench)
    benchmark_java_parser(bench)
    benchmark_multi_language(bench)

    # API benchmarks (if server is running)
    print("\n📊 Benchmarking API endpoints...")
    benchmark_api_endpoints(bench)

    # Print results
    bench.print_results()

    # Export to JSON
    output_dir = Path(__file__).parent / "results"
    output_dir.mkdir(exist_ok=True)

    timestamp = time.strftime('%Y%m%d_%H%M%S')
    output_file = output_dir / f"benchmark_{timestamp}.json"
    bench.export_json(str(output_file))

    # Summary statistics
    print("\n📈 SUMMARY STATISTICS")
    print("-" * 100)

    parser_results = [r for r in bench.results if 'Parser' in r.name]
    if parser_results:
        avg_parser_time = statistics.mean([r.mean_ms for r in parser_results])
        print(f"Average Parser Time: {avg_parser_time:.2f}ms")

    api_results = [r for r in bench.results if 'API' in r.name]
    if api_results:
        avg_api_time = statistics.mean([r.mean_ms for r in api_results])
        print(f"Average API Time: {avg_api_time:.2f}ms")

    print("\n✅ Benchmark suite completed successfully!")


if __name__ == "__main__":
    main()
