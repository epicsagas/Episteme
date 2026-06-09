/**
 * Bilingual help content for dashboard sections and cards.
 * Structure: { en: { title, lines }, ko: { title, lines } }
 * Lines support HTML: <strong>, <code>, <span style="...">.
 */
export const help = {
  compositeTrend: {
    en: {
      title: 'Composite Score Trend',
      lines: [
        'Shows how overall quality metrics change across runs.',
        '<strong>composite</strong> — weighted average: <code>0.3×recall + 0.3×precision + 0.2×specificity + 0.2×smell_recall</code>. Range 0.0–1.0, ≥0.8 is good.',
        '<strong>recall</strong> — fraction of actual relevant results found by search. Higher = fewer misses.',
        '<strong>precision</strong> — fraction of search results that are relevant. Higher = less noise.',
        '<strong>specificity</strong> — fraction of negative queries that returned no relevant IDs. Higher = fewer false positives.',
        '<strong>smell_recall</strong> — fraction of actual code smells detected. Higher = fewer missed smells.',
      ],
    },
    ko: {
      title: 'Composite Score 추이',
      lines: [
        '실행별 전체 품질 지표의 변화를 보여줍니다.',
        '<strong>composite</strong> — 가중 평균: <code>0.3×recall + 0.3×precision + 0.2×specificity + 0.2×smell_recall</code>. 0.0~1.0, 0.8 이상 양호.',
        '<strong>recall</strong> — 검색이 실제 정답을 찾아낸 비율. 높을수록 누락이 적음.',
        '<strong>precision</strong> — 검색 결과 중 정답의 비율. 높을수록 오탐이 적음.',
        '<strong>specificity</strong> — 부정 쿼리에서 정답을 반환하지 않은 비율. 높을수록 거짓 양성이 적음.',
        '<strong>smell_recall</strong> — 실제 코드 스멜 중 탐지된 비율. 높을수록 놓치는 스멜이 적음.',
      ],
    },
  },

  composite: {
    en: {
      title: 'Composite Score',
      lines: [
        'Weighted average of all quality dimensions.',
        '<code>0.3×recall + 0.3×precision + 0.2×specificity + 0.2×smell_recall</code>',
        'Range 0.0–1.0. <strong>≥0.8</strong> is good, <strong><0.6</strong> needs improvement.',
        'Regression threshold: <code>≥0.02</code> drop vs previous run → FAIL.',
      ],
    },
    ko: {
      title: 'Composite Score',
      lines: [
        '모든 품질 차원의 가중 평균입니다.',
        '<code>0.3×recall + 0.3×precision + 0.2×specificity + 0.2×smell_recall</code>',
        '0.0~1.0 범위. <strong>0.8 이상</strong> 양호, <strong>0.6 미만</strong> 개선 필요.',
        '회귀 탐지 기준: 이전 실행 대비 <code>≥0.02</code> 하락 시 FAIL.',
      ],
    },
  },

  recall: {
    en: {
      title: 'Recall',
      lines: [
        'Fraction of <strong>actual relevant results found</strong> by search.',
        'Example: 10 relevant items exist, 8 found → recall = 0.8.',
        'Low recall means relevant results are being missed. Consider expanding the index or adjusting weights.',
      ],
    },
    ko: {
      title: 'Recall (재현율)',
      lines: [
        '검색이 <strong>실제 정답 중 몇 개를 찾았는지</strong> 비율.',
        '예: 정답이 10개인데 8개를 찾으면 recall = 0.8.',
        '낮으면 관련 결과를 누락하고 있다는 뜻. 인덱스 확장이나 가중치 조정 필요.',
      ],
    },
  },

  precision: {
    en: {
      title: 'Precision',
      lines: [
        'Fraction of search results that are <strong>actually relevant</strong>.',
        'Example: 5 results returned, 4 relevant → precision = 0.8.',
        'Low precision means too many irrelevant results. Consider refining queries or improving ranking.',
      ],
    },
    ko: {
      title: 'Precision (정밀도)',
      lines: [
        '검색 결과 중 <strong>실제 정답의 비율</strong>.',
        '예: 5개 결과 중 4개가 정답이면 precision = 0.8.',
        '낮으면 관련 없는 결과가 많다는 뜻. 쿼리 정제나 랭킹 개선 필요.',
      ],
    },
  },

  specificity: {
    en: {
      title: 'Specificity',
      lines: [
        'Fraction of negative queries that <strong>did not return relevant IDs</strong>.',
        'Example: querying "Python" should not return Rust entities. If it doesn\'t, specificity is high.',
        'Low specificity means irrelevant results are being returned. Adjust thresholds.',
      ],
    },
    ko: {
      title: 'Specificity (특이도)',
      lines: [
        '부정 쿼리에서 <strong>정답 ID를 반환하지 않은 비율</strong>.',
        '예: "Python" 쿼리에 Rust 결과가 안 나오면 specificity 높음.',
        '낮으면 관련 없는 결과를 반환하는 것. 임계값 조정 필요.',
      ],
    },
  },

  smellRecall: {
    en: {
      title: 'Smell Recall',
      lines: [
        'Fraction of <strong>actual code smells detected</strong> by the analyzer.',
        'Example: 20 smells exist, 16 detected → 0.8.',
        'Low smell recall means the detector is missing smells. Consider adding or refining detection rules.',
      ],
    },
    ko: {
      title: 'Smell Recall (스멜 탐지율)',
      lines: [
        '실제 코드 스멜 중 <strong>탐지된 비율</strong>.',
        '예: 20개 스멜 중 16개 탐지 → 0.8.',
        '낮으면 탐지기가 놓치는 스멜이 많음. 규칙 보완 필요.',
      ],
    },
  },

  searchPositive: {
    en: {
      title: 'Search Positive',
      lines: [
        'Tests search quality on queries that <strong>have a known correct answer</strong>.',
        '<strong>hit@1</strong> — Is the first result correct? (1=hit, 0=miss).',
        '<strong>hit@3 / hit@5</strong> — Is the correct answer in the top 3/5 results?',
        '<strong>RR (MRR@5)</strong> — Reciprocal Rank: inverse of the correct answer\'s rank. 1st=1.0, 2nd=0.5, 3rd=0.33…',
        '<strong>NDCG@5</strong> — Normalized Discounted Cumulative Gain. Closer to <code>1.0</code> = better ranking.',
        'Row colors: <span style="color:var(--red)">red</span>=hit@1 miss, <span style="color:var(--yellow)">yellow</span>=RR<1.0. Click a row for details.',
      ],
    },
    ko: {
      title: 'Search Positive',
      lines: [
        '정답이 있는 쿼리로 검색 품질을 측정합니다.',
        '<strong>hit@1</strong> — 첫 번째 결과가 정답인지 (1=적중, 0=실패).',
        '<strong>hit@3 / hit@5</strong> — 상위 3/5개 중 정답 포함 여부.',
        '<strong>RR (MRR@5)</strong> — Reciprocal Rank. 정답 순위의 역수: 1위=1.0, 2위=0.5, 3위=0.33…',
        '<strong>NDCG@5</strong> — Normalized DCG. 순위에 따라 할인된 적재 이득. <code>1.0</code>에 가까울수록 이상적.',
        '행 색상: <span style="color:var(--red)">빨강</span>=hit@1 실패, <span style="color:var(--yellow)">노랑</span>=RR<1.0. 행 클릭 시 상세 모달.',
      ],
    },
  },

  searchPositiveMetrics: {
    en: {
      title: 'Search Positive Metrics',
      lines: [
        'Averaged search performance across queries with known answers.',
        '<strong>hit@1</strong> — Probability the first result is correct. Most important metric.',
        '<strong>hit@3/5</strong> — Probability the answer appears in top 3/5.',
        '<strong>RR</strong> — Mean Reciprocal Rank. 1.0 = always ranked first.',
        '<strong>NDCG</strong> — Ranking-aware precision. 1.0 = perfect ordering.',
      ],
    },
    ko: {
      title: 'Search Positive 지표',
      lines: [
        '정답이 존재하는 쿼리들의 검색 성능 평균입니다.',
        '<strong>hit@1</strong> — 첫 결과가 정답일 확률. 가장 중요한 지표.',
        '<strong>hit@3/5</strong> — 상위 3/5개에 정답이 있을 확률.',
        '<strong>RR</strong> — 정답 순위의 역수 평균. 1.0=항상 1위.',
        '<strong>NDCG</strong> — 순위를 고려한 정밀도. 1.0=완벽한 순서.',
      ],
    },
  },

  searchNegative: {
    en: {
      title: 'Search Negative',
      lines: [
        'Tests that search <strong>does not return</strong> results for queries with no correct answer.',
        '<strong>FP@1/3/5</strong> — False Positive rate at top 1/3/5. <code>0%</code> is ideal.',
        '<strong>specificity</strong> — Fraction of negative queries returning no relevant IDs.',
        'Red rows indicate violations — the search returned an entity it shouldn\'t have.',
      ],
    },
    ko: {
      title: 'Search Negative',
      lines: [
        '정답이 없는 쿼리에서 검색이 <strong>결과를 반환하지 않는지</strong> 테스트합니다.',
        '<strong>FP@1/3/5</strong> — 상위 1/3/5개에서 거짓 양성 비율. <code>0%</code>가 이상적.',
        '<strong>specificity</strong> — 부정 쿼리 중 정답을 반환하지 않은 비율.',
        '빨간 행은 위반 — 검색이 반환하지 말아야 할 엔티티를 반환함.',
      ],
    },
  },

  searchNegativeFp: {
    en: {
      title: 'False Positive Rates',
      lines: [
        '<strong>FP@1</strong> — How often the top result is a false positive.',
        '<strong>FP@3/5</strong> — How often top 3/5 contain a false positive.',
        'All should be as close to <code>0%</code> as possible.',
      ],
    },
    ko: {
      title: '거짓 양성 비율',
      lines: [
        '<strong>FP@1</strong> — 첫 결과가 거짓 양성인 비율.',
        '<strong>FP@3/5</strong> — 상위 3/5개에 거짓 양성이 포함된 비율.',
        '모두 <code>0%</code>에 가까워야 합니다.',
      ],
    },
  },

  searchNegativeSpecificity: {
    en: {
      title: 'Specificity (Negative Queries)',
      lines: [
        '<strong>specificity</strong> — Overall rate of correctly returning no results for negative queries.',
        '<strong>true_negatives</strong> — Count of queries that returned no false positives.',
        '<strong>total</strong> — Total number of negative queries.',
      ],
    },
    ko: {
      title: '특이도 (부정 쿼리)',
      lines: [
        '<strong>specificity</strong> — 부정 쿼리에서 정확히 결과를 반환하지 않은 전체 비율.',
        '<strong>true_negatives</strong> — 거짓 양성을 반환하지 않은 쿼리 수.',
        '<strong>total</strong> — 전체 부정 쿼리 수.',
      ],
    },
  },

  smellNegative: {
    en: {
      title: 'Smell Negative (False Positive Rate)',
      lines: [
        'Tests that smell detectors <strong>do not flag</strong> clean code as having smells.',
        '<strong>fp_rate</strong> — Overall false positive rate. <code>0%</code> is ideal.',
        '<strong>Per Detector</strong> — FP rate broken down by each detector. Identify which detectors over-trigger.',
        '<strong>Per Language</strong> — FP rate broken down by programming language.',
        'Green = 0%, Yellow = <50%, Red = ≥50%.',
      ],
    },
    ko: {
      title: 'Smell Negative (거짓 양성률)',
      lines: [
        '스멜 탐지기가 <strong>깨끗한 코드에 스멜이 있다고 오탐하지 않는지</strong> 테스트합니다.',
        '<strong>fp_rate</strong> — 전체 거짓 양성률. <code>0%</code>가 이상적.',
        '<strong>탐지기별</strong> — 각 탐지기별 거짓 양성률. 어떤 탐지기가 과다 반응하는지 파악.',
        '<strong>언어별</strong> — 프로그래밍 언어별 거짓 양성률.',
        '초록 = 0%, 노랑 = <50%, 빨강 = ≥50%.',
      ],
    },
  },

  smellNegativeFp: {
    en: {
      title: 'FP Rate Details',
      lines: [
        '<strong>fp_rate</strong> — Overall false positive rate across all clean code samples.',
        '<strong>fp_count</strong> — Number of false positives detected.',
        '<strong>total</strong> — Total number of clean code samples tested.',
        '<strong>specificity</strong> — Complement of FP rate (1 − fp_rate).',
      ],
    },
    ko: {
      title: 'FP 비율 상세',
      lines: [
        '<strong>fp_rate</strong> — 깨끗한 코드 샘플 전체의 거짓 양성률.',
        '<strong>fp_count</strong> — 탐지된 거짓 양성 수.',
        '<strong>total</strong> — 테스트된 깨끗한 코드 샘플 총수.',
        '<strong>specificity</strong> — FP 비율의 여집합 (1 − fp_rate).',
      ],
    },
  },

  analyzePositive: {
    en: {
      title: 'Analyze Positive (Smell Detection Recall)',
      lines: [
        'Tests whether the analyzer <strong>correctly detects</strong> known code smells.',
        '<strong>recall</strong> — Fraction of actual smells that were detected.',
        '<strong>Per Smell Recall</strong> — Breakdown by smell type. Identifies which smells are hardest to detect.',
        'Bars closer to 100% mean better detection for that smell type.',
      ],
    },
    ko: {
      title: 'Analyze Positive (스멜 탐지율)',
      lines: [
        '분석기가 <strong>실제 코드 스멜을 정확히 탐지하는지</strong> 테스트합니다.',
        '<strong>recall</strong> — 실제 스멜 중 탐지된 비율.',
        '<strong>스멜별 탐지율</strong> — 스멜 유형별 분석. 어떤 스멜이 탐지하기 가장 어려운지 파악.',
        '막대가 100%에 가까울수록 해당 스멜 탐지 성능이 좋음.',
      ],
    },
  },

  analyzePositiveRecall: {
    en: {
      title: 'Detection Recall',
      lines: [
        '<strong>recall</strong> — Fraction of known smells successfully detected.',
        '<strong>hits</strong> — Number of smells correctly identified.',
        '<strong>total</strong> — Total number of known smell instances.',
      ],
    },
    ko: {
      title: '탐지 재현율',
      lines: [
        '<strong>recall</strong> — 알려진 스멜 중 성공적으로 탐지된 비율.',
        '<strong>hits</strong> — 올바르게 식별된 스멜 수.',
        '<strong>total</strong> — 알려진 스멜 인스턴스 총수.',
      ],
    },
  },

  traversal: {
    en: {
      title: 'Traversal',
      lines: [
        'Tests the graph traversal capabilities — finding connected entities in the knowledge graph.',
        '<strong>Neighbors</strong> — Can it find directly connected entities (function calls, type references, etc.)?',
        '<strong>Paths</strong> — Can it find multi-hop paths between two entities?',
        'Both use <strong>recall</strong>: fraction of expected results that were actually found.',
      ],
    },
    ko: {
      title: 'Traversal',
      lines: [
        '지식 그래프에서 연결된 엔티티를 찾는 그래프 순회 능력을 테스트합니다.',
        '<strong>Neighbors</strong> — 직접 연결된 엔티티(함수 호출, 타입 참조 등)를 찾는지?',
        '<strong>Paths</strong> — 두 엔티티 간 다중 홉 경로를 찾는지?',
        '둘 다 <strong>recall</strong> 사용: 예상 결과 중 실제로 찾은 비율.',
      ],
    },
  },

  traversalNeighbors: {
    en: {
      title: 'Neighbors Recall',
      lines: [
        '<strong>recall</strong> — Fraction of expected neighbor entities correctly found.',
        '<strong>hits</strong> — Number of neighbors correctly returned.',
        '<strong>total</strong> — Total expected neighbor count.',
        'Tests direct relationships: function→caller, type→field, module→import, etc.',
      ],
    },
    ko: {
      title: '인접 노드 재현율',
      lines: [
        '<strong>recall</strong> — 예상된 인접 엔티티 중 올바르게 찾은 비율.',
        '<strong>hits</strong> — 올바르게 반환된 인접 노드 수.',
        '<strong>total</strong> — 예상된 인접 노드 총수.',
        '직접 관계 테스트: 함수→호출자, 타입→필드, 모듈→임포트 등.',
      ],
    },
  },

  traversalPaths: {
    en: {
      title: 'Paths Recall',
      lines: [
        '<strong>recall</strong> — Fraction of expected multi-hop paths correctly found.',
        '<strong>hits</strong> — Number of paths correctly returned.',
        '<strong>total</strong> — Total expected path count.',
        'Tests graph traversal depth: can it find A→B→C when only A and C are given?',
      ],
    },
    ko: {
      title: '경로 재현율',
      lines: [
        '<strong>recall</strong> — 예상된 다중 홉 경로 중 올바르게 찾은 비율.',
        '<strong>hits</strong> — 올바르게 반환된 경로 수.',
        '<strong>total</strong> — 예상된 경로 총수.',
        '그래프 순회 깊이 테스트: A와 C만 주어졌을 때 A→B→C를 찾는지?',
      ],
    },
  },
};
