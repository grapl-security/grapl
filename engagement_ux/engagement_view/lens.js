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

const mapNodeProps = (node, f) => {
    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if(Array.isArray(node[prop])) {
                if (node[prop].length > 0) {
                    if (node[prop][0].uid === undefined) {
                        f(prop)
                    }
                }
            } else {
                f(prop)
            }
        }
    }
};

const mapEdgeProps = (node, f) => {
    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if(Array.isArray(node[prop])) {
                for (const neighbor of node[prop]) {
                    if (neighbor.uid !== undefined) {
                        f(prop, neighbor)
                    }
                }
            }
        }
    }
};

const _mapGraph = (node, visited, f) => {
    if (visited.has(node.uid)) {
        return
    }
    visited.add(node.uid);
    mapEdgeProps(node, (edgeName, neighbor) => {
        f(node, edgeName, neighbor);
        _mapGraph(neighbor, visited, f)
    })
};

const mapGraph = (node, f) => {
  const visited = new Set();
  mapEdgeProps(node, (edgeName, neighbor) => {
      f(node, edgeName, neighbor);
      _mapGraph(neighbor, visited, f)
  })
};


const edgeLinksFromNode = (node) => {
    const links = [];

    if (node.lens) { return [] }

    mapEdgeProps(node, (edgeName, targetNode) => {
        const target = targetNode.uid;
        links.push({
            source: node.uid,
            target,
        });
    });
    return links;
};


const lensToAdjacencyMatrix = (lens) => {
    console.log('lensNode', lens);
    const nodes = new Map();
    const links = new Map();

    mapGraph(lens, (fromNode, edgeName, toNode) => {
        nodes.set(fromNode.uid, fromNode);
        nodes.set(toNode.uid, toNode);
        let edgeList = links.get(fromNode.uid);
        if (edgeList === undefined) {
            edgeList = new Map();
            edgeList.set(
                fromNode.uid + edgeName + toNode.uid,
                [fromNode.uid, edgeName, toNode.uid]
            );

        } else {
            edgeList.set(
                fromNode.uid + edgeName + toNode.uid,
                [fromNode.uid, edgeName, toNode.uid]
            );
        }
        links.set(fromNode.uid, edgeList)
    });

    return {
        nodes, links
    }
};

const dgraphNodesToD3Format = (dgraphNodes) => {
    const graph = lensToAdjacencyMatrix(dgraphNodes[0]);

    // Calculate risks and attach to nodes
    for (const node of graph.nodes.values()) {
        const edges = graph.links.get(node.uid) || new Map();
        for (const edge of edges.values()) {
            if (edge[1] === 'risks') {
                const riskNode = graph.nodes.get(edge[2]);
                if (!riskNode.risk_score) {
                    continue
                }
                if (node.risk === undefined) {
                    node.risk = riskNode.risk_score
                    node.analyzers = riskNode.analyzer_name
                } else {
                    node.risk += riskNode.risk_score
                    if (node.analyzers.indexOf(riskNode.analyzer_name) === -1) {
                        node.analyzers += ', ' + riskNode.analyzer_name
                    }
                }
            }
        }
    }

    // Flatten nodes
    const nodes = [];

    for (const node of graph.nodes.values()) {
        console.log('node', node);
        if (node.risk_score || node.analyzer_name) {
            continue
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
    }

    // Flatten links
    const links = [];

    for (const linkMap_ of graph.links) {
        const nodeId = linkMap_[0];
        const node = graph.nodes.get(nodeId);

        if (node && node.lens) {
            // Don't link lens nodes, it's ugly
            continue
        }
        const linkMap = linkMap_[1];
        for (const links_ of linkMap.values()) {
            if (links_[1] === 'risks') {
                continue
            }
            if (links_[1] === '~scope') {
                continue
            }

            if (links_[1][0] === '~') {
                links.push({
                    source: links_[2],
                    label: links_[1].substr(1),
                    target: links_[0],
                });
            } else {
                links.push({
                    source: links_[0],
                    label: links_[1],
                    target: links_[2],
                });
            }

        }
    }

    console.log('links', links);


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
    const hidden = new Set(['risks','uid', 'scope', 'name', 'nodeType', 'nodeLabel', 'x', 'y', 'index', 'vy', 'vx', 'fx', 'fy']);
    mapEdgeProps(node, (edgeName, neighbor) => {
        hidden.add(edgeName)
    })

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



const buf2hex = (buffer) => { // buffer is an ArrayBuffer
    return Array.prototype.map.call(new Uint8Array(buffer), x => ('00' + x.toString(16)).slice(-2)).join('');
};

const hashNode = async (node) => {
    let nodeStr = "" + node.uid;

    const props = [];

    mapNodeProps(node, (prop) => {
        props.push(prop + node[prop]);
    });

    props.sort();
    nodeStr += props.join("");

    const edges = [];

    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if(Array.isArray(node[prop])) {
                const edgeUids = [];
                for (const neighbor of node[prop]) {
                    if (neighbor.uid !== undefined) {
                        edgeUids.push(prop + neighbor.uid);
                    }
                }

                edgeUids.sort();
                edges.push(edgeUids.join(""))
            }
        }
    }

    edges.sort();
    nodeStr += edges.join("");

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

        console.log('removed_nodes', removed_nodes);
        if (updated_nodes.length !== 0) {
            graphManager.updateGraph(dgraphNodesToD3Format(updated_nodes));
        }

        if (removed_nodes.length !== 0) {
            // graphManager.removeNodesAndLinks(removed_nodes);
        }
    } catch (e) {
        console.warn("Failed to fetch updates ", e)
    }

    graphManager.update();
    // setTimeout(async () => {
    //     await updateLoop(graphManager, lens);
    //     graphManager.update();
    // }, 1000)
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
