<template>
    <div class="benchmark-chart-container" :style="{ height: containerHeight }">
        <canvas ref="chartCanvas"></canvas>
    </div>
</template>

<script setup>
import { ref, computed, onMounted, watch, onUnmounted } from "vue";
import { Chart, registerables } from "chart.js";

Chart.register(...registerables);

const props = defineProps({
    data: {
        type: Array,
        required: true,
    },
    unit: {
        type: String,
        default: "seconds",
    },
    title: {
        type: String,
        default: "",
    },
});

const chartCanvas = ref(null);
let chartInstance = null;

const containerHeight = computed(() => {
    const barHeight = 40;
    const titleHeight = props.title ? 50 : 0;
    const axisHeight = 40;
    const padding = 32;
    return `${props.data.length * barHeight + titleHeight + axisHeight + padding}px`;
});

const brandGreen = "#3c8d58";
const brandGreenLight = "#47a467";
const mutedGray = "#6b7280";
const mutedGrayLight = "#9ca3af";

const getColors = (isDark) => {
    return props.data.map((item) =>
        item.highlight
            ? isDark
                ? brandGreenLight
                : brandGreen
            : isDark
              ? mutedGrayLight
              : mutedGray,
    );
};

const createChart = () => {
    if (!chartCanvas.value) return;

    const isDark = document.documentElement.classList.contains("dark");
    const textColor = isDark ? "#e5e7eb" : "#374151";
    const gridColor = isDark
        ? "rgba(255, 255, 255, 0.1)"
        : "rgba(0, 0, 0, 0.1)";

    const labels = props.data.map((item) => item.label);
    const values = props.data.map((item) => item.value);
    const colors = getColors(isDark);

    if (chartInstance) {
        chartInstance.destroy();
    }

    chartInstance = new Chart(chartCanvas.value, {
        type: "bar",
        data: {
            labels,
            datasets: [
                {
                    data: values,
                    backgroundColor: colors,
                    borderColor: colors,
                    borderWidth: 1,
                    borderRadius: 4,
                    barThickness: 28,
                    maxBarThickness: 28,
                    minBarLength: 8,
                },
            ],
        },
        options: {
            indexAxis: "y",
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    display: false,
                },
                title: {
                    display: !!props.title,
                    text: props.title,
                    color: textColor,
                    font: {
                        size: 16,
                        weight: "600",
                    },
                    padding: {
                        bottom: 16,
                    },
                },
                tooltip: {
                    callbacks: {
                        label: (context) => {
                            const value = context.raw;
                            if (props.unit === "mb") {
                                return value >= 1000
                                    ? `${(value / 1000).toFixed(2)} GB`
                                    : `${value} MB`;
                            }
                            if (props.unit === "million") {
                                return `~${value.toFixed(1)} Million`;
                            }
                            // seconds
                            if (value < 1) {
                                return `${(value * 1000).toFixed(1)}ms`;
                            }
                            return `${value.toFixed(2)}s`;
                        },
                    },
                },
            },
            scales: {
                x: {
                    beginAtZero: true,
                    grid: {
                        color: gridColor,
                    },
                    ticks: {
                        color: textColor,
                        callback: (value) => {
                            if (props.unit === "mb") {
                                return value >= 1000
                                    ? `${(value / 1000).toFixed(1)}GB`
                                    : `${value}MB`;
                            }
                            if (props.unit === "million") {
                                return `${value}M`;
                            }
                            // seconds
                            if (value < 1) {
                                return `${(value * 1000).toFixed(0)}ms`;
                            }
                            return `${value}s`;
                        },
                    },
                    title: {
                        display: true,
                        text:
                            props.unit === "mb"
                                ? "Memory"
                                : props.unit === "million"
                                  ? "Cycles"
                                  : "Time",
                        color: textColor,
                    },
                },
                y: {
                    grid: {
                        display: false,
                    },
                    ticks: {
                        color: textColor,
                        font: {
                            size: 13,
                            weight: "500",
                        },
                    },
                },
            },
        },
    });
};

let observer = null;

onMounted(() => {
    createChart();

    observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
            if (mutation.attributeName === "class") {
                createChart();
            }
        });
    });

    observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["class"],
    });
});

onUnmounted(() => {
    if (chartInstance) {
        chartInstance.destroy();
    }
    if (observer) {
        observer.disconnect();
    }
});

watch(() => props.data, createChart, { deep: true });
</script>

<style scoped>
.benchmark-chart-container {
    width: 100%;
    margin: 1.5rem 0;
    padding: 1rem;
    background: var(--vp-c-bg-soft);
    border-radius: 8px;
}
</style>
