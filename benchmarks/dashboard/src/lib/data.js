/**
 * Data loading and transformation layer for the eval dashboard.
 * Extracted from App.svelte to decouple data from view.
 */

/**
 * Load all eval runs from build-time JSON imports.
 * @param {Record<string, any>} rawFiles - import.meta.glob result
 * @returns {Array<{ filename: string; timestamp: Date; label: string; git_commit: string; composite_score: object; regression: object; suites: object }>}
 */
export function loadEvalRuns(rawFiles) {
  return Object.entries(rawFiles)
    .filter(([path]) => {
      const name = path.split('/').at(-1);
      return name.startsWith('eval_') && name.endsWith('.json');
    })
    .map(([path, mod]) => {
      const filename = path.split('/').at(-1);
      const data = mod.default ?? mod;
      return {
        filename,
        timestamp: new Date(data.timestamp),
        label: filename.replace('eval_', '').replace('.json', ''),
        git_commit: data.git_commit,
        composite_score: data.composite_score,
        regression: data.regression,
        suites: data.suites,
      };
    })
    .sort((a, b) => a.timestamp - b.timestamp);
}

/**
 * Derive trend data (one point per run) for line charts.
 */
export function toTrendRuns(runs) {
  return runs.map((r) => ({
    label: r.label,
    timestamp: r.timestamp,
    composite: r.composite_score.composite,
    recall: r.composite_score.recall,
    precision: r.composite_score.precision,
    specificity: r.composite_score.specificity,
    smell_recall: r.composite_score.smell_recall,
  }));
}

/**
 * Build per-metric full-history series for sparklines.
 */
export function toSeries(trendRuns) {
  return {
    composite: trendRuns.map((r) => r.composite),
    recall: trendRuns.map((r) => r.recall),
    precision: trendRuns.map((r) => r.precision),
    specificity: trendRuns.map((r) => r.specificity),
    smell_recall: trendRuns.map((r) => r.smell_recall),
  };
}

/**
 * Compute per-metric deltas between current and previous run.
 */
export function computeDeltas(currentScore, prevScore) {
  if (!currentScore) return {};
  const d = (curr, prev) => (prev == null ? null : curr - prev);
  return {
    composite: d(currentScore.composite, prevScore?.composite),
    recall: d(currentScore.recall, prevScore?.recall),
    precision: d(currentScore.precision, prevScore?.precision),
    specificity: d(currentScore.specificity, prevScore?.specificity),
    smell_recall: d(currentScore.smell_recall, prevScore?.smell_recall),
  };
}
