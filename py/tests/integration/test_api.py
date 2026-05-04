#!/usr/bin/env python3
"""
Test script for Syntagma REST API
Validates all endpoints with sample data
"""

import requests
import json
import time
import pytest

BASE_URL = "http://localhost:8000"


def _api_available() -> bool:
    try:
        requests.get(f"{BASE_URL}/health", timeout=2)
        return True
    except requests.exceptions.ConnectionError:
        return False


pytestmark = pytest.mark.skipif(not _api_available(), reason="Syntagma API server not running")

# Sample Python code with smells
SAMPLE_CODE = '''
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

    return total
'''


def test_health():
    """Test health endpoint"""
    print("\n=== Testing /health ===")
    response = requests.get(f"{BASE_URL}/health")
    print(f"Status: {response.status_code}")
    print(json.dumps(response.json(), indent=2))
    assert response.status_code == 200
    assert response.json()["status"] == "healthy"


def test_stats():
    """Test stats endpoint"""
    print("\n=== Testing /stats ===")
    response = requests.get(f"{BASE_URL}/stats")
    print(f"Status: {response.status_code}")
    print(json.dumps(response.json(), indent=2))
    assert response.status_code == 200


def test_analyze():
    """Test code smell detection"""
    print("\n=== Testing /analyze ===")
    response = requests.post(
        f"{BASE_URL}/analyze", json={"code": SAMPLE_CODE, "min_confidence": 0.5}
    )
    print(f"Status: {response.status_code}")
    result = response.json()
    print(f"Smells detected: {result['smells_detected']}")

    for detection in result["detections"]:
        print(f"\n  - {detection['smell_name']} (confidence: {detection['confidence']:.2f})")
        print(f"    Location: {detection['location']}")
        print(
            f"    Metrics: LOC={detection['metrics']['loc']}, CC={detection['metrics']['cyclomatic_complexity']}"
        )

    assert response.status_code == 200
    assert result["smells_detected"] > 0


def test_refactor():
    """Test refactoring suggestions"""
    print("\n=== Testing /refactor ===")
    response = requests.post(
        f"{BASE_URL}/refactor", json={"code": SAMPLE_CODE, "top_k": 3, "min_confidence": 0.5}
    )
    print(f"Status: {response.status_code}")
    result = response.json()
    print(f"Smells analyzed: {result['smells_analyzed']}")

    for analysis in result["results"]:
        smell = analysis["smell"]
        print(f"\n  Smell: {smell['smell_name']} (confidence: {smell['confidence']:.2f})")

        suggestions = analysis["suggestions"]
        if suggestions:
            print(f"  Suggestions:")
            for i, sug in enumerate(suggestions, 1):
                print(
                    f"    {i}. {sug['title']} (priority: {sug['priority_score']:.2f}, effort: {sug['effort']})"
                )

    assert response.status_code == 200


def test_search():
    """Test semantic search"""
    print("\n=== Testing /search ===")
    response = requests.post(
        f"{BASE_URL}/search", json={"query": "How to fix Long Method?", "top_k": 5}
    )
    print(f"Status: {response.status_code}")
    result = response.json()
    print(f"Results: {result['results_count']}")

    for i, res in enumerate(result["results"][:3], 1):
        print(f"\n  {i}. {res['title']} ({res['entity_id']})")
        print(f"     Similarity: {res['similarity']:.4f}")
        print(f"     Type: {res['entity_type']}")

    assert response.status_code == 200
    assert result["results_count"] > 0


def test_graph_entity():
    """Test get entity"""
    print("\n=== Testing /graph/{entity_id} ===")
    response = requests.get(f"{BASE_URL}/graph/DP-005")
    print(f"Status: {response.status_code}")
    result = response.json()
    print(f"Entity: {result.get('title', 'Unknown')} ({result.get('id')})")
    print(f"Type: {result.get('type')}")
    print(f"Relations: {list(result.get('relations', {}).keys())}")

    assert response.status_code == 200


def test_graph_neighbors():
    """Test get neighbors"""
    print("\n=== Testing /graph/neighbors ===")
    response = requests.post(
        f"{BASE_URL}/graph/neighbors", json={"entity_id": "SMELL-01", "relation_type": "solved_by"}
    )
    print(f"Status: {response.status_code}")
    result = response.json()
    print(f"Neighbors: {result['neighbor_count']}")

    for neighbor in result["neighbors"]:
        print(f"  - {neighbor['id']}: {neighbor['title']} ({neighbor['type']})")

    assert response.status_code == 200


def test_graph_path():
    """Test shortest path"""
    print("\n=== Testing /graph/path ===")
    response = requests.post(
        f"{BASE_URL}/graph/path", json={"from_id": "SMELL-01", "to_id": "LAW-042-S", "max_depth": 5}
    )
    print(f"Status: {response.status_code}")
    result = response.json()

    if result["path_found"]:
        print(f"Path found: {result['hops']} hops")
        for node in result["path"]:
            print(f"  -> {node['id']}: {node['title']}")
    else:
        print("No path found")

    assert response.status_code == 200


def test_graph_subgraph():
    """Test subgraph extraction"""
    print("\n=== Testing /graph/subgraph ===")
    response = requests.post(
        f"{BASE_URL}/graph/subgraph", json={"center_id": "DP-005", "radius": 2}
    )
    print(f"Status: {response.status_code}")
    result = response.json()
    print(f"Nodes: {result['node_count']}, Edges: {result['edge_count']}")

    assert response.status_code == 200


def test_performance():
    """Test API performance"""
    print("\n=== Performance Test ===")

    tests = [
        ("Health Check", lambda: requests.get(f"{BASE_URL}/health")),
        (
            "Analyze",
            lambda: requests.post(
                f"{BASE_URL}/analyze", json={"code": SAMPLE_CODE, "min_confidence": 0.5}
            ),
        ),
        (
            "Search",
            lambda: requests.post(f"{BASE_URL}/search", json={"query": "Long Method", "top_k": 5}),
        ),
        ("Graph Entity", lambda: requests.get(f"{BASE_URL}/graph/DP-005")),
    ]

    for name, test_func in tests:
        start = time.time()
        response = test_func()
        latency = (time.time() - start) * 1000
        print(f"{name:20} - {response.status_code} - {latency:6.1f}ms")


def main():
    """Run all tests"""
    print(f"Testing Syntagma API at {BASE_URL}")
    print("=" * 60)

    try:
        # Check if server is running
        response = requests.get(f"{BASE_URL}/")
        print(f"Server is running (version: {response.json()['version']})")
    except requests.exceptions.ConnectionError:
        print(f"Server is not running at {BASE_URL}")
        print("   Start server with: uvicorn api.main:app --reload")
        return

    # Run tests
    tests = [
        test_health,
        test_stats,
        test_analyze,
        test_refactor,
        test_search,
        test_graph_entity,
        test_graph_neighbors,
        test_graph_path,
        test_graph_subgraph,
        test_performance,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            test()
            passed += 1
        except Exception as e:
            print(f"\nTest failed: {e}")
            failed += 1

    print("\n" + "=" * 60)
    print(f"Results: {passed} passed, {failed} failed")

    if failed == 0:
        print("All tests passed!")
    else:
        print("Some tests failed")


if __name__ == "__main__":
    main()
