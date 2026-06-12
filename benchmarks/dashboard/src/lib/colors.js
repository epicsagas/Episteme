/**
 * Consistent color mapping for all dashboard metrics.
 * Composite = bright white-blue, Recall = blue, Precision = green,
 * Specificity = amber, Smell Recall = red.
 */
export const METRIC_COLORS = {
  composite: '#e6edf3',
  recall: '#4a9eff',
  precision: '#66bb6a',
  specificity: '#ffd54f',
  smell_recall: '#ef5350',
};

/** Bar chart default color for smell-negative per-detector */
export const BAR_CHART_RED = '#ef5350';

/** Bar chart color for analyze-positive per-smell-recall */
export const BAR_CHART_GREEN = '#66bb6a';
