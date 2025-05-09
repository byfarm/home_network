import * as d3 from "https://cdn.jsdelivr.net/npm/d3@7/+esm";

const dataset = document.currentScript.dataset;

const data = [
    { date: new Date("2023-02-15"), value: 45 },
    { date: new Date("2023-04-10"), value: 62 },
    { date: new Date("2023-06-25"), value: 58 },
    { date: new Date("2023-08-01"), value: 75 },
    { date: new Date("2023-10-18"), value: 88 },
    { date: new Date("2023-12-05"), value: 95 }
];
dataset.data = data;

// Declare the chart dimensions and margins.
const width = 640;
const height = 400;
const marginTop = 20;
const marginRight = 20;
const marginBottom = 30;
const marginLeft = 40;

// Declare the x (horizontal position) scale.
const x = d3.scaleUtc()
    .domain([dataset.xMin, dataset.xMax])
    .range([marginLeft, width - marginRight]);

// Declare the y (vertical position) scale.
const y = d3.scaleLinear()
    .domain([dataset.yMin, dataset.yMax])
    .range([height - marginBottom, marginTop]);

// Create the SVG container.
const svg = d3.create("svg")
    .attr("width", width)
    .attr("height", height);

// Add the x-axis.
svg.append("g")
    .attr("transform", `translate(0,${height - marginBottom})`)
    .call(d3.axisBottom(x));

// Add the y-axis.
svg.append("g")
    .attr("transform", `translate(${marginLeft},0)`)
    .call(d3.axisLeft(y));

const line = d3.line()
    .x(d => x(d.date)) // Use the x scale for date
    .y(d => y(d.value)); // Use the y scale for value

svg.append("path") // Append a path element for the line
    .datum(dataset.data) // Bind the *entire* data array to the path
    .attr("fill", "none") // No fill for the line
    .attr("stroke", "green") // Set line color
    .attr("stroke-width", 1.5) // Set line thickness
    .attr("d", line); // Generate the 'd' attribute using the line generator

// insert into the dom
const targetTag = document.querySelector(dataset.targetReplaceTag)
if (targetTag) {
    targetTag.replaceWith(svg.node())
} else {
    console.error("Could not find image element.");
}
