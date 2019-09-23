// Stylesheets
console.log('entry.js init');

const engagement_edge = "";

if (engagement_edge.length === 0) {
    console.assert("Engagement Edge URL can not be empty. Run build.sh");
}

console.log(`Connecting to ${engagement_edge}`);

class GraphManager {
    constructor(graph) {
        this.canvas = d3.select('#network');
        this.width = this.canvas.attr('width');
        this.height = this.canvas.attr('height');
        this.ctx = this.canvas.node().getContext('2d');
        this.r = 20;
        this.color = d3.scaleOrdinal(d3.schemeCategory10);
        this.simulation = d3.forceSimulation()
            .force("x", d3.forceX(this.width/2))
            .force("y", d3.forceY(this.height/2))
            .force('collide', d3.forceCollide(this.r * 4))
            // .force('charge', d3.forceManyBody()
            //     .strength(-40))
            .force('link', d3.forceLink()
                .id(d => d.uid));


        this.simulation.nodes(graph.nodes);

        this.simulation.force('link')
            .links(graph.links);

        this.simulation.on('tick', () => {
            this.update();
        });

        this.canvas
            .call(d3.drag()
                .container(this.canvas.node())
                .subject(this.dragsubject)
                .on('start', this.dragstarted)
                .on('drag', this.dragged)
                .on('end', this.dragended));


        this.tooltip = d3.select('body')
            .append('div')
            .style('position', 'absolute')
            .style('z-index', '100')
            .html('<text id="tooltip">Click a node to see its attributes</text>');

        this.graph = graph;
    }

    updateNode = (newNode) => {
        if (newNode.uid === undefined) {return}
        for (let node of this.graph.nodes) {
            if (node.name === newNode.name) {
                node = newNode;
                return;
            }
        }
        this.graph.nodes.push(newNode);
    };

    updateLink(newLink) {
        for (const link of this.graph.links) {
            if (link.source === newLink.source) {
                if (link.target === newLink.target) {
                    return;
                }
            }
        }
        this.graph.links.push(newLink);
    }

    removeNode = (uid) => {
        for (let i = 0; i < this.graph.nodes.length; i++) {
            console.log('checking', this.graph.nodes[i].uid);
            console.log('checking against', uid);
            if (this.graph.nodes[i].uid === uid) {
                console.log("Removing node");
                this.graph.nodes.splice(i, 1);
            }
        }
    };

    removeLink = (uid) => {
        for (let i = 0; i < this.graph.links.length; i++) {
            if (this.graph.links[i].source.uid === uid) {
                this.graph.links.splice(i, 1);
                continue
            }
            if (this.graph.links[i].target.uid === uid) {
                this.graph.links.splice(i, 1);
            }
        }
    };

    removeNodesAndLinks = (toRemove) => {
        console.log("Removing ", toRemove);
        for (const deadNode of toRemove) {
            this.removeNode(deadNode);
        }
        this.simulation.nodes(this.graph.nodes);

        for (const deadLink of toRemove) {
            this.removeLink(deadLink);
        }

        this.simulation.force('link')
            .links(this.graph.links);

        console.log("Removed nodes and links ", this.graph.nodes, this.graph.links);
        this.update();
    };

    updateGraph = (newGraph) => {
        for (const newNode of newGraph.nodes) {
            this.updateNode(newNode);
        }
        this.simulation.nodes(this.graph.nodes);

        for (const newLink of newGraph.links) {
            this.updateLink(newLink);
        }

        this.simulation.force('link')
            .links(this.graph.links);

        this.update();
    };

    update = () => {
        this.ctx.clearRect(0, 0, this.width, this.height);

        this.simulation.nodes(this.graph.nodes);
        this.simulation.force('link')
            .links(this.graph.links);

        this.ctx.beginPath();
        this.ctx.globalAlpha = 1.0;
        this.ctx.strokeStyle = '#aaa';
        this.graph.links.forEach(this.drawLink);
        this.ctx.stroke();

        this.graph.nodes.forEach(d => this.drawNode(d));

    };

    dragsubject = () => this.simulation.find(d3.event.x, d3.event.y);


    dragstarted = () => {
        if (!d3.event.active) this.simulation.alphaTarget(0.3).restart();
        d3.event.subject.fx = d3.event.subject.x;
        d3.event.subject.fy = d3.event.subject.y;

        this.start_x = d3.event.subject.x;
        this.start_y = d3.event.subject.y;
    };

    dragged = () => {
        d3.event.subject.fx = d3.event.x;
        d3.event.subject.fy = d3.event.y;
    };

    dragended = () => {
        if (!d3.event.active) this.simulation.alphaTarget(0);
        d3.event.subject.fx = null;
        d3.event.subject.fy = null;

        if (d3.event.subject.x === this.start_x) {
            if (d3.event.subject.y === this.start_y) {
                const node = d3.event.subject;

                const t = document.getElementById('tooltip');
                if (t !== null) {
                    t.remove();
                }


                const s = nodeToTable(node);

                this.tooltip = d3.select('body')
                    .append('div')
                    .style('position', 'absolute')
                    .style('z-index', '100')
                    .style('visibility', 'hidden')
                    .html(`
                        <table class="table" id="tooltip">
                            ${s}
                        </table>
                    `);

                this.tooltip.style('visibility', 'visible');
            }
        }

        this.start_x = null;
        this.start_y = null;
    };

    drawNode = (d) => {
        this.ctx.beginPath();
        this.ctx.fillStyle = this.color(d.nodeType);
        this.ctx.moveTo(d.x, d.y);
        this.ctx.arc(d.x, d.y, this.r, 0, Math.PI * 2);
        this.ctx.fill();

        this.ctx.fillStyle = 'black';
        this.ctx.font = '16px Arial';
        this.ctx.fillText(d.nodeLabel, d.x - this.r, d.y + 35);
    };

    drawLink = (l) => {
        this.ctx.moveTo(l.source.x, l.source.y);
        this.ctx.lineTo(l.target.x, l.target.y);
    };
}

const edgeNames = [
    'children',
    'bin_file',
    'created_files',
    'scope',
];

const edgeLinksFromNode = (node) => {
    const links = [];

    if (node.lens) { return [] }

    for (const edgeName of edgeNames) {
        if (node[edgeName] !== undefined) {
            for (const targetNode of node[edgeName]) {
                const target = targetNode.uid;
                links.push({
                    source: node.uid,
                    target,
                });
            }
        }
    }
    return links;
};

const getNeighbors = (node) => {
    const neighbors = [];

    for (const edgeName of edgeNames) {
        if (node[edgeName] !== undefined) {
            for (const targetNode of node[edgeName]) {
                const targetNodeType = getNodeType(targetNode);
                const targetNodeLabel = getNodeLabel(targetNodeType, targetNode);
                console.log('target', targetNode);
                console.log('targetNodeType', targetNodeType);
                console.log('targetNodeLabel', targetNodeLabel);

                if(targetNode.risks !== undefined) {
                    console.log('targetNode.risks ', targetNode.risks);
                    let score = 0;
                    const risks = [];

                    for (const risk of targetNode.risks) {
                        score += risk.risk_score;
                        risks.push(risk.analyzer_name);
                    }
                    targetNode.score = score;
                    targetNode.risk_name = risks;
                    delete targetNode.risks
                }

                neighbors.push({
                    ...targetNode,
                    nodeType: targetNodeType,
                    nodeLabel: targetNodeLabel,
                });
            }
        }
    }

    return neighbors
};

const dgraphNodesToD3Format = (dgraphNodes) => {
    const nodes = [];
    const links = [];

    for (const node of dgraphNodes) {
        if (node.uid === undefined) {
            console.log('Skipping undefiend node ' + JSON.stringify(node));
            continue
        }

        if(node.risks !== undefined) {
            node.score = node.risks.score;
            node.risk_name = node.risks.analyzer_name;
            delete node.risks
        }

        const nodeType = getNodeType(node);
        const nodeLabel = getNodeLabel(nodeType, node);
        nodes.push({
            name: node.uid,
            ...node,
            nodeType,
            nodeLabel,
            x: 200 + randomInt(1, 50),
            y: 150 + randomInt(1, 50),
        });

        const neighbors = getNeighbors(node);

        for (const neighbor of neighbors) {
            nodes.push({
                name: neighbor.uid,
                ...neighbor,
                x: 200 + randomInt(1, 50),
                y: 150 + randomInt(1, 50),
            });

            const nextNeighbors = getNeighbors(neighbor);

            for (const nextNeighbor in nextNeighbors) {
                nodes.push({
                    name: nextNeighbor.uid,
                    ...nextNeighbor,
                    x: 200 + randomInt(1, 50),
                    y: 150 + randomInt(1, 50),
                });
            }

            edgeLinksFromNode(neighbor).forEach(link => links.push(link));
        }

        edgeLinksFromNode(node).forEach(link => links.push(link));
    }

    return {
        nodes,
        links,
    };
};

const getNodeLabel = (nodeType, node) => {
    if (nodeType === 'Process') {
        return node.process_name || node.process_id;
    }

    if (nodeType === 'File') {
        return node.file_path;
    }

    if (nodeType === 'ExternalIp') {
        return node.external_ip;
    }

    if (nodeType === 'Connect') {
        return node.port;
    }

    if (nodeType === 'Lens') {
        return node.lens;
    }

    return '';
};

const getNodeType = (node) => {
    if (node.process_id !== undefined) {
        return 'Process';
    }
    if (node.file_path !== undefined) {
        return 'File';
    }

    if (node.external_ip !== undefined) {
        return 'ExternalIp';
    }

    if (node.port !== undefined) {
        return 'Connect';
    }

    if (node.scope !== undefined) {
        return 'Lens';
    }

    console.warn('Unable to find type for node');
    return 'Unknown';
};


const nodeToTable = (node) => {
    const hidden = new Set(['uid', 'scope', 'name', 'nodeType', 'nodeLabel', 'x', 'y', 'index', 'vy', 'vx', 'fx', 'fy']);
    edgeNames.forEach((name) => hidden.add(name));

    let header = '<thead class="thead"><tr>';
    let output = '<tbody><tr>';

    for (const [field, value] of Object.entries(node)) {
        if (hidden.has(field) || node.uid === undefined) {
            continue
        }

        header += `<th scope="col">${field}</th>`;

        if (field.includes('_time')) {
            output += `<td>${new Date(value).toLocaleString()}</td>>`;
        } else {
            output += `<td>${value}</td>>`;
        }

    }

    return `${header}</tr></thead>` + `${output}</tr><tbody>`;
};


const processProperties = [
    'process_id', 'node_key', 'create_time', 'arguments',
    'process_name'
];

const fileProperties = [
    'process_id', 'node_key', 'file_path'
];


const buf2hex = (buffer) => { // buffer is an ArrayBuffer
    return Array.prototype.map.call(new Uint8Array(buffer), x => ('00' + x.toString(16)).slice(-2)).join('');
};

const hashNode = async (node) => {
    let nodeStr = "" + node.uid;

    if (node.nodeType === "Process") {
        for (const prop of processProperties) {
            nodeStr += node[prop] || ''
        }
    }

    if (node.nodeType === "File") {
        for (const prop of fileProperties) {
            nodeStr += node[prop] || ''
        }
    }

    for (const edge of edgeNames) {
        if (node[edge] !== undefined) {
            nodeStr += edge + node[edge].length
        } else {
            nodeStr += edge + '0'
        }
    }

    // return nodeStr
    return buf2hex(await window.crypto.subtle.digest(
        "SHA-256",
        new TextEncoder().encode(nodeStr)
    ));
};

const retrieveGraph = async (graph, lens) => {

    let uidHashes = {};

    for (const node of graph.nodes) {
        if (node.lens !== undefined) {
            continue
        }
        if (node.uid !== undefined) {
            uidHashes[node.uid] = await hashNode(node);
        }
    }

    console.log("Getting graph");

    const res = await fetch(`${engagement_edge}update`, {
        method: 'post',
        body: JSON.stringify({
            'lens': lens,
            'uid_hashes': uidHashes,
        })
    });

    const json_res = await res.json();
    console.info('jsonres ' + json_res);
    const updated_nodes = json_res['updated_nodes'];
    const removed_nodes = json_res['removed_nodes'];

    return [updated_nodes, removed_nodes]
};


const updateLoop = async (graphManager, lens) => {
    try {
        console.info('Fetching updates');
        const [updated_nodes, removed_nodes] = await retrieveGraph(
            graphManager.graph, lens
        );

        if (updated_nodes.length !== 0) {
            graphManager.updateGraph(dgraphNodesToD3Format(updated_nodes));
        }

        if (removed_nodes.length !== 0) {
            graphManager.removeNodesAndLinks(removed_nodes);
        }
    } catch (e) {
        console.warn("Failed to fetch updates ", e)
    }

    setTimeout(async () => {
        await updateLoop(graphManager, lens);
        graphManager.update();
    }, 1000)
};

function randomInt(min, max) // min and max included
{
    return Math.floor(Math.random() * (max - min + 1) + min);
}

document.addEventListener('DOMContentLoaded', async (event) => {
    console.log('DOMContentLoaded');
    const lens = new URLSearchParams(window.location.search).get('lens');

    if (lens === null || lens.length <= 0) {
        console.error('Failed to retrieve egId from url');
        return;
    }

    // console.log("Initializing graphManager with, ", initGraph);
    const graphManager = new GraphManager(
        {nodes: [], links: []},
    );
    console.log("Starting update loop");
    await updateLoop(graphManager, lens);
});
