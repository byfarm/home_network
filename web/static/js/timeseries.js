const dataset = document.currentScript.dataset;

const parseDateTime = d3.timeParse("%Y-%m-%d %H:%M:%S");

var data = []
function appendToData(x, y) {
    data.push({
        date: parseDateTime(x),
        value: y
    });
}

window.chartData.x.forEach((element, i) => {
    appendToData(element, window.chartData.y[i]);
});


// Declare the chart dimensions and margins.
const width = 640;
const height = 400;
const marginTop = 20;
const marginRight = 20;
const marginBottom = 30;
const marginLeft = 40;

// Declare the x (horizontal position) scale.
const x = d3.scaleUtc()
    .domain(d3.extent(data, d => d.date))
    .range([marginLeft, width - marginRight]);

// Declare the y (vertical position) scale.
const y = d3.scaleLinear()
    .domain(d3.extent(data, d => d.value))
    .range([height - marginBottom, marginTop]);

// Create the SVG container.
const svg = d3.create("svg")
    .attr("width", width)
    .attr("height", height);

// Add the x-axis.
const xAxis = svg.append("g")
    .attr("transform", `translate(0,${height - marginBottom})`)
    .call(d3.axisBottom(x));

// Add the y-axis.
const yAxis = svg.append("g")
    .attr("transform", `translate(${marginLeft},0)`)
    .call(d3.axisLeft(y));

const line = d3.line()
    .x(d => x(d.date)) // Use the x scale for date
    .y(d => y(d.value)); // Use the y scale for value

const path = svg.append("path") // Append a path element for the line
    .datum(data) // Bind the *entire* data array to the path
    .attr("fill", "none") // No fill for the line
    .attr("stroke", "green") // Set line color
    .attr("stroke-width", 1.5) // Set line thickness
    .attr("d", d => { // Add logging here to see the generated path data string
        const pathData = line(d);
        console.log("Generated path data (d attribute):", pathData);
        return pathData;
    });

function insertGraph() {
    const targetTag = document.querySelector(dataset.target_replace_tag)
    if (targetTag) {
        targetTag.replaceWith(svg.node())
    } else {
        console.error("Could not find image element.");
    }
}
insertGraph()
