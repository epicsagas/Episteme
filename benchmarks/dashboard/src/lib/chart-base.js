/**
 * Shared Chart.js module registration.
 * Import this once per component instead of calling Chart.register() individually.
 */
import {
  Chart,
  LineController, LineElement, PointElement,
  BarController, BarElement,
  LinearScale, CategoryScale,
  Tooltip, Legend,
} from 'chart.js';

// Register all needed modules in one place
Chart.register(
  LineController, LineElement, PointElement,
  BarController, BarElement,
  LinearScale, CategoryScale,
  Tooltip, Legend,
);

export { Chart, Tooltip };
