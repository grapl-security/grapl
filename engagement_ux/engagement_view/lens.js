// Stylesheets
console.log('entry.js init');
const engagement_edge = "http://localhost:8900/";

if (engagement_edge.length === 0) {
    console.assert("Engagement Edge URL can not be empty. Run build.sh");
}

console.log(`Connecting to ${engagement_edge}`);

const BKDRHash = (str) => {
    const seed = 131;
    const seed2 = 137;
    let hash = 0;
    // make hash more sensitive for short string like 'a', 'b', 'c'
    str += 'x';
    // Note: Number.MAX_SAFE_INTEGER equals 9007199254740991
    const MAX_SAFE_INTEGER = parseInt(9007199254740991 / seed2);
    for(let i = 0; i < str.length; i++) {
        if(hash > MAX_SAFE_INTEGER) {
            hash = parseInt(hash / seed2);
        }
        hash = hash * seed + str.charCodeAt(i);
    }
    return hash;
};


/**
 * Convert HSL to RGB
 *
 * @see {@link http://zh.wikipedia.org/wiki/HSL和HSV色彩空间} for further information.
 * @param {Number} H Hue ∈ [0, 360)
 * @param {Number} S Saturation ∈ [0, 1]
 * @param {Number} L Lightness ∈ [0, 1]
 * @returns {Array} R, G, B ∈ [0, 255]
 */
const HSL2RGB = (H, S, L) => {
    H /= 360;

    const q = L < 0.5 ? L * (1 + S) : L + S - L * S;
    const p = 2 * L - q;

    return [H + 1/3, H, H - 1/3].map((color) => {
        if(color < 0) {
            color++;
        }
        if(color > 1) {
            color--;
        }
        if(color < 1/6) {
            color = p + (q - p) * 6 * color;
        } else if(color < 0.5) {
            color = q;
        } else if(color < 2/3) {
            color = p + (q - p) * 6 * (2/3 - color);
        } else {
            color = p;
        }
        return Math.round(color * 255);
    });
};

const isArray = (o) => {
    return Object.prototype.toString.call(o) === '[object Array]';
};

/**
 * Color Hash Class
 *
 * @class
 */
const ColorHash = function(options) {
    options = options || {};

    const LS = [options.lightness, options.saturation].map((param) => {
        param = param || [0.35, 0.5, 0.65]; // note that 3 is a prime
        return isArray(param) ? param.concat() : [param];
    });

    this.L = LS[0];
    this.S = LS[1];

    if (typeof options.hue === 'number') {
        options.hue = {min: options.hue, max: options.hue};
    }
    if (typeof options.hue === 'object' && !isArray(options.hue)) {
        options.hue = [options.hue];
    }
    if (typeof options.hue === 'undefined') {
        options.hue = [];
    }
    this.hueRanges = options.hue.map(function (range) {
        return {
            min: typeof range.min === 'undefined' ? 0 : range.min,
            max: typeof range.max === 'undefined' ? 360: range.max
        };
    });

    this.hash = options.hash || BKDRHash;
};

/**
 * Returns the hash in [h, s, l].
 * Note that H ∈ [0, 360); S ∈ [0, 1]; L ∈ [0, 1];
 *
 * @param {String} str string to hash
 * @returns {Array} [h, s, l]
 */
ColorHash.prototype.hsl = function(str) {
    let H, S, L;
    let hash = this.hash(str);

    if (this.hueRanges.length) {
        const range = this.hueRanges[hash % this.hueRanges.length];
        const hueResolution = 727; // note that 727 is a prime
        H = ((hash / this.hueRanges.length) % hueResolution) * (range.max - range.min) / hueResolution + range.min;
    } else {
        H = hash % 359; // note that 359 is a prime
    }
    hash = parseInt(hash / 360);
    S = this.S[hash % this.S.length];
    hash = parseInt(hash / this.S.length);
    L = this.L[hash % this.L.length];

    return [H, S, L];
};

/**
 * Returns the hash in [r, g, b].
 * Note that R, G, B ∈ [0, 255]
 *
 * @param {String} str string to hash
 * @returns {Array} [r, g, b]
 */
ColorHash.prototype.rgb = function(str) {
    const hsl = this.hsl(str);
    return HSL2RGB.apply(this, hsl);
};



const graph3d = (elem) => {
    const viz = ForceGraph3D()(elem)
        .enableNodeDrag(true)
        .onNodeHover(node => elem.style.cursor = node ? 'pointer' : null)
        .graphData({nodes: [], links: []})
        .nodeLabel(node => node.nodeLabel)
        .linkCurvature('curvature')
        .nodeAutoColorBy('nodeType')
        .linkThreeObjectExtend(true)
        .nodeThreeObjectExtend(true)
        .linkOpacity(0.5)
        .linkDirectionalArrowLength(2)
        .linkDirectionalArrowRelPos(1.8)
        .linkThreeObject(link => {
            const sprite = new SpriteText(mapLabel(link.label));
            sprite.color = 'cyan';
            sprite.textHeight = 3;

            return sprite;
        })
        .linkPositionUpdate((sprite, { start, end }) => {
            const middlePos = Object.assign(...['x', 'y', 'z'].map(c => ({
                [c]: start[c] + (end[c] - start[c]) / 2 // calc middle point
            })));
            // Position sprite
            Object.assign(sprite.position, middlePos);
        })
        .nodeThreeObject(node => {
            // use a sphere as a drag handle
            const obj = new THREE.Mesh(
                new THREE.SphereGeometry(4),
                new THREE.MeshBasicMaterial({ depthWrite: false, transparent: false, opacity: 1 })
            );

            // add text sprite as child
            const sprite = new SpriteText('   ' + node.nodeLabel + '   ');

            sprite.color = 'red';
            sprite.textHeight = 4;
            obj.add(sprite);
            return obj;
        });

    // viz.d3Force('charge').strength(-220);
    return viz
};


const mapLabel = (label) => {
    if (label === 'children') {
        return 'executed'
    }
    return label
};


const percentToColor = (percentile) => {
    const hue = (100 - percentile) * 40 / 100;

    return `hsl(${hue}, 100%, 50%)`;
};


const calcNodeRiskPercentile = (_nodeRisk, _allRisks) => {
    let nodeRisk = _nodeRisk;
    if (typeof _nodeRisk === 'object') {
        nodeRisk = _nodeRisk.risk;
    }

    const allRisks = _allRisks
        .map(n => n || 0)
        .sort((a, b) => a - b);

    if (nodeRisk === undefined || nodeRisk === 0 || allRisks.length === 0) {
        return 0
    }

    let riskIndex = 0;
    for (const risk of allRisks) {
        if (nodeRisk >= risk) {
            riskIndex += 1;
        }
    }

    return Math.floor((riskIndex / allRisks.length) * 100)
};


const nodeSize = (node, Graph) => {
    const nodes = [...Graph.graphData().nodes].map(node => node.risk);
    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile >= 75) {
        return 6
    } else if (riskPercentile >= 25) {
        return 5
    } else {
        return 4
    }
};


const riskColor = (node, Graph, colorHash) => {
    const nodes = [...Graph.graphData().nodes].map(node => node.risk);

    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile === 0) {
        const nodeColors = calcNodeRgb(node, colorHash);
        return `rgba(${nodeColors[0]}, ${nodeColors[1]}, ${nodeColors[2]}, 1)`;
    }

    return percentToColor(riskPercentile);
};

const calcLinkRisk = (link, Graph) => {
    let srcNode = findNode(link.source, Graph.graphData().nodes)
        || findNode(link.source.name, Graph.graphData().nodes);
    let dstNode = findNode(link.target, Graph.graphData().nodes)
        || findNode(link.target.name, Graph.graphData().nodes);

    if (srcNode === null) {
        srcNode = {risk: 0}
    }

    if (dstNode === null) {
        dstNode = {risk: 0}
    }

    const srcRisk = srcNode.risk || 0;
    const dstRisk = dstNode.risk || 0;

    return Math.round((srcRisk + dstRisk) / 2)
};


const calcLinkRiskPercentile = (link, Graph) => {
    const linkRisk = calcLinkRisk(link, Graph);
    const nodes = [...Graph.graphData().nodes].map(node => node.risk);

    return calcNodeRiskPercentile(linkRisk, nodes);
};


const calcLinkParticleWidth = (link, Graph) => {
    const linkRiskPercentile = calcLinkRiskPercentile(link, Graph);
    if (linkRiskPercentile >= 75) {
        return 5
    } else if (linkRiskPercentile >= 50) {
        return 4
    }  else if (linkRiskPercentile >= 25) {
        return 3
    } else {
        return 2
    }
};


const calcLinkColor = (link, Graph) => {
    const risk = calcLinkRiskPercentile(link, Graph);
    if (risk === 0) {return undefined}
    return percentToColor(risk);
};


const calcNodeRgb = (node, colorHash) => {
    if (node.nodeType === 'Process') {
        return [50, 153, 169]
    }

    if (node.nodeType === 'File') {
        return [89, 180, 39]
    }

    return colorHash.rgb(node.nodeType)
}

const sanitizeHTML = (str) => {
    const temp = document.createElement('div');
    temp.textContent = str;
    return temp.innerHTML;
};

const findNode = (id, nodes) => {
    for (const node of (nodes || [])) {
        if (node.id === id) {
            return node
        }
    }
    return null
};

/*
* Determines where the DirectionalArrow lies on the edge, based on
* the inferred size of the target node (where size is itself determined
* by risk)
* */
const calcLinkDirectionalArrowRelPos = (link, Graph) => {
    const node = findNode(link.target, Graph.graphData().nodes)
        || findNode(link.target.name, Graph.graphData().nodes);

    if (node === null || node.risk === 0) {
        return 1.0
    }
    const nodes = [...Graph.graphData().nodes].map(node => node.risk);
    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile === 0) {return 1.0}

    if (riskPercentile >= 75) {
        return 0.95
    } else if (riskPercentile >= 50) {
        return 0.9
    } else if (riskPercentile >= 25) {
        return 0.85
    } else {
        return 1.0
    }
};

const graph2d = (elem) => {
    const colorHash = new ColorHash();

    const Graph = ForceGraph()(elem)
        .graphData({nodes: [], links: []})
        .onNodeHover(node => {
            // highlightNodes = node ? [node] : [];
            elem.style.cursor = node ? '-webkit-grab' : null;
        })
        .linkDirectionalParticles(1)
        .linkDirectionalParticleWidth((link) => {
            return calcLinkParticleWidth(link, 
                document.getElementById('graph'));
        })
        .linkDirectionalParticleColor((link) => {
            return calcLinkColor(link, Graph)
        })
        .linkDirectionalParticleSpeed(0.005)
        .linkWidth(4)
        .linkAutoColorBy((link) => {
            return calcLinkColor(link, Graph)
        })
        .linkDirectionalArrowLength(8)
        // .linkDirectionalArrowColor(link => {
        //     return'rgba(323,421,543,224)'
                 // })
        .linkDirectionalArrowRelPos(link => {
            return calcLinkDirectionalArrowRelPos(link, Graph);
        })
        .linkCanvasObjectMode(() => 'after')
        .linkCanvasObject((link, ctx) => {
            const MAX_FONT_SIZE = 8;
            const LABEL_NODE_MARGIN = Graph.nodeRelSize() * 1.5;
            const start = link.source;
            const end = link.target;
            // ignore unbound links
            link.color = calcLinkColor(link, Graph);

            if (typeof start !== 'object' || typeof end !== 'object') return;

            // calculate label positioning
            const textPos = Object.assign(...['x', 'y'].map(c => ({
                [c]: start[c] + (end[c] - start[c]) / 2 // calc middle point
            })));
            const relLink = { x: end.x - start.x, y: end.y - start.y };

            const maxTextLength = Math.sqrt(Math.pow(relLink.x, 2) + Math.pow(relLink.y, 2)) - LABEL_NODE_MARGIN * 8;

            let textAngle = Math.atan2(relLink.y, relLink.x);
            // maintain label vertical orientation for legibility
            if (textAngle > Math.PI / 2) textAngle = -(Math.PI - textAngle);
            if (textAngle < -Math.PI / 2) textAngle = -(-Math.PI - textAngle);
            const label = mapLabel(link.label);
            // estimate fontSize to fit in link length
            ctx.font = '2px Sans-Serif';
            const fontSize = Math.min(MAX_FONT_SIZE, maxTextLength / ctx.measureText(label).width);
            ctx.font = `${fontSize + 5}px Sans-Serif`;

            let textWidth = ctx.measureText(label).width;

            textWidth += Math.round(textWidth * 0.25);

            const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding
            // draw text label (with background rect)
            ctx.save();
            ctx.translate(textPos.x, textPos.y);
            ctx.rotate(textAngle);
            ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
            ctx.fillRect(- bckgDimensions[0] / 2, - bckgDimensions[1] / 2, ...bckgDimensions);
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillStyle = 'black';
            ctx.fillText(label, 0, 0);
            ctx.restore();
        })
        .nodeCanvasObject((node, ctx, globalScale) => {
            // add ring just for highlighted nodes
            const NODE_R = nodeSize(node, Graph);
            ctx.save();

            // Risk outline color
            ctx.beginPath();
            ctx.arc(node.x, node.y, NODE_R * 1.3, 0, 2 * Math.PI, false);
            ctx.fillStyle = riskColor(node, Graph, colorHash);
            ctx.fill();
            ctx.restore();

            ctx.save();

            // Node color
            ctx.beginPath();
            ctx.arc(node.x, node.y, NODE_R * 1.2, 0, 2 * Math.PI, false);

            const nodeRbg = calcNodeRgb(node, colorHash);

            ctx.fillStyle = `rgba(${nodeRbg[0]}, ${nodeRbg[1]}, ${nodeRbg[2]}, 1)`;
            ctx.fill();
            ctx.restore();

            const label = node.nodeLabel;

            const fontSize = 15/globalScale;

            ctx.font = `${fontSize}px Sans-Serif`;

            const textWidth = ctx.measureText(label).width;

            const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding

            ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
            ctx.fillRect(node.x - bckgDimensions[0] / 2, node.y - bckgDimensions[1] / 2, ...bckgDimensions);
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillStyle = 'rgba(0, 0, 0, 0.8)';
            ctx.fillText(label, node.x, node.y);

        })
        .onNodeClick(node => {
            const table = (document.getElementById('nodes'));

            const s = nodeToTable(node, Graph);

            table.innerHTML = `
                <div>
                    <table class="boldtable" id="node">
                        ${s}
                    </table>
                </div>
                
            `;

        })
        .onNodeRightClick(node => {
            // Right click expands node
            // Pulls edges/ properties down, but does not copy over to the engagement graph
            // Increased opacity on 'phantom' nodes
        })
        .d3VelocityDecay(0.75)
    ;

    Graph.d3Force("link", d3.forceLink());
    Graph.d3Force('collide', d3.forceCollide(22));

    Graph.d3Force("charge", d3.forceManyBody());

    Graph.d3Force('box', () => {
        const N = 100;
        // console.log(Graph.width(), Graph.height())
        const SQUARE_HALF_SIDE = 20 * N * 0.5;
        Graph.graphData().nodes.forEach(node => {
            const x = node.x || 0, y = node.y || 0;
            // bounce on box walls
            if (Math.abs(x) > SQUARE_HALF_SIDE) { node.vx *= -1; }
            if (Math.abs(y) > SQUARE_HALF_SIDE) { node.vy *= -1; }
        });
    });
    return Graph

};

// merges y into x, returns true if update occurred
const mergeNodes = (x, y) => {
    let merged = false;
    mapNodeProps(y, (prop) => {
        if (!Object.prototype.hasOwnProperty.call(x, prop)) {
            merged = true;
            x[prop] = y[prop]
        }
    });

    return merged;
};


class GraphManager {
    constructor(graph, dimension) {
        const elem = document.getElementById("graph");

        if (dimension === '3d') {
            this.viz = graph3d(elem)
        } else if (dimension === '2d') {
            this.viz = graph2d(elem);
        }
        else {
            this.viz = graph2d(elem);
        }

        this.graph = {...graph};
    }

    updateNode = (newNode) => {
        if (newNode.uid === undefined) {return}
        for (let node of this.graph.nodes) {
            if (node.name === newNode.name) {
                return mergeNodes(node, newNode);
            }
        }
        console.log('adding new node');
        this.graph.nodes.push(newNode);
        return true;
    };

    updateLink(newLink) {
        for (const link of this.graph.links) {
            let src = link.source.name;
            if (src === undefined) {
                src = link.source;
            }

            let tgt = link.target.name;
            if (tgt === undefined) {
                tgt = link.target;
            }

            if (src === newLink.source) {
                if (tgt === newLink.target) {
                    // if (link.label === newLink.label) {

                    return false;
                    // }
                }
            }
        }

        this.graph.links.push(newLink);
        return true;
    }

    removeNode = (uid) => {
        for (let i = 0; i < this.graph.nodes.length; i++) {
            if (this.graph.nodes[i].uid === uid) {
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
        for (const deadNode of toRemove) {
            this.removeNode(deadNode);
        }

        for (const deadLink of toRemove) {
            this.removeLink(deadLink);
        }

        // console.log("Removed nodes and links ", this.graph.nodes, this.graph.links);
    };

    updateGraph = (newGraph) => {
        if (newGraph.nodes.length === 0 && newGraph.links.length === 0) {
            return
        }

        if (newGraph === this.graph) {
            return
        }

        let updated = false;
        for (const newNode of newGraph.nodes) {
            if (this.updateNode(newNode)) {
                updated = true;
            }
        }

        for (const newLink of newGraph.links) {
            if (this.updateLink(newLink)) {
                updated = true;
            }
        }

        if (updated) {
            this.update();
        }
    };

    update = () => {
        this.viz.graphData({...this.graph})

    };

}
            }
        }

        this.graph.links.push(newLink);
        return true;
    }

    removeNode = (uid) => {
        for (let i = 0; i < this.graph.nodes.length; i++) {
            if (this.graph.nodes[i].uid === uid) {
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
        for (const deadNode of toRemove) {
            this.removeNode(deadNode);
        }

        for (const deadLink of toRemove) {
            this.removeLink(deadLink);
        }

        // console.log("Removed nodes and links ", this.graph.nodes, this.graph.links);
    };

    updateGraph = (newGraph) => {
        if (newGraph.nodes.length === 0 && newGraph.links.length === 0) {
            return
        }

        if (newGraph === this.graph) {
            return
        }

        let updated = false;
        for (const newNode of newGraph.nodes) {
            if (this.updateNode(newNode)) {
                updated = true;
            }
        }

        for (const newLink of newGraph.links) {
            if (this.updateLink(newLink)) {
                updated = true;
            }
        }

        if (updated) {
            this.update();
        }
    };

    update = () => {
        this.viz.graphData({...this.graph})

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
    mapEdgeProps(node, (edgeName, neighbor) => {
        if (visited.has(node.uid + edgeName + neighbor.uid)) {
            return
        }

        visited.add(node.uid + edgeName + neighbor.uid);

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
            curvature: 2
        });
    });
    return links;
};


const lensToAdjacencyMatrix = (matricies) => {
    const nodes = new Map();
    const links = new Map();

    const key_uid = new Map();

    for (const matrix of matricies) {
        const node_key = matrix.node['node_key'];
        const uid = matrix.node['uid'];
        if (matrix.node["analyzer_name"]) {
            continue
        }
        key_uid.set(node_key, uid);
        console.log(node_key);
        nodes.set(uid, matrix.node);
    }

    for (const matrix of matricies) {
        const node_key = matrix.node['node_key'];
        const uid = matrix.node['uid'];

        for (const edge of matrix.edges) {
            let edgeList = links.get(uid);
            const to_uid = key_uid.get(edge['to']);

            const edge_name = edge['edge_name'];
            if (edge_name === "risks") {
                const node = nodes.get(key_uid.get(edge['from']));
                for (const risk of matrix.node.risks) {
                    node.risk = risk.risk_score + (node.risk || 0)
                    if (node.analyzers) {
                        if (node.analyzers.indexOf(risk.analyzer_name) === -1) {
                            node.analyzers += ', ' + risk.analyzer_name
                        }
                    } else {
                        node.analyzers = risk.analyzer_name
                    }
                }

                continue
            }

            if (edgeList === undefined) {
                edgeList = new Map();
                edgeList.set(
                    uid +  + to_uid,
                    [uid, edge_name, to_uid]
                );

            } else {
                edgeList.set(
                    uid + edge_name + to_uid,
                    [uid, edge_name, to_uid]
                );
            }
            links.set(uid, edgeList)
        }
    }

    console.log(links)

    return {
        nodes, links
    }
};

const dgraphNodesToD3Format = (dgraphNodes) => {
    const graph = lensToAdjacencyMatrix(dgraphNodes);

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
                    node.risk = riskNode.risk_score;
                    node.analyzers = riskNode.analyzer_name;
                } else {
                    node.risk += riskNode.risk_score;
                    if (node.analyzers && node.analyzers.indexOf(riskNode.analyzer_name) === -1) {
                        node.analyzers += ', ' + riskNode.analyzer_name;
                    }
                }
            }
        }
    }

    // Flatten nodes
    const nodes = [];

    for (const node of graph.nodes.values()) {
        if (node.risk_score || node.analyzer_name) {
            continue
        }
        const nodeType = getNodeType(node);
        if (nodeType === 'Unknown') {
            continue
        }
        const nodeLabel = getNodeLabel(nodeType, node);
        nodes.push({
            name: node.uid,
            id: node.uid,
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

    return node.node_type || '';
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

    if (node.scope !== undefined || node.lens !== undefined) {
        return 'Lens';
    }

    // Dynamic nodes
    if (node.node_type) {
        return node.node_type
    }

    console.warn('Unable to find type for node ', node);
    return 'Unknown';
};


const nodeToTable = (node, Graph) => {
    const hidden = new Set(['id', 'dgraph.type', '__indexColor', 'risks','uid', 'scope', 'name', 'nodeType', 'nodeLabel', 'x', 'y', 'index', 'vy', 'vx', 'fx', 'fy']);
    mapEdgeProps(node, (edgeName, _neighbor) => {
        hidden.add(edgeName)
    });

    let header = '<thead class="thead"><tr>';
    let output = '<tbody><tr>';

    for (const [field, value] of Object.entries(node)) {
        if (hidden.has(field) || node.uid === undefined) {
            continue
        }

        header += `<th scope="col">${field}</th>`;

        if (field.includes('_time')) {
            try {
                output += `<td>${new Date(value).toLocaleString()}</td>`;
            } catch (e) {
                console.warn('Could not convert timestamp: ', e);
                output += `<td>${sanitizeHTML(value)}</td>`;
            }
        } else {
            if (value.length > 128) {
                output += `<td>${sanitizeHTML(value.slice(0, 25))}</td>`;
            } else {
                output += `<td>${sanitizeHTML(value)}</td>`;
            }
        }
    }

    header += `<th scope="col">risk %</th>`;
    const nodes = [...Graph.graphData().nodes].map(node => node.risk);
    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);
    output += `<td>${sanitizeHTML(riskPercentile)}</td>`;

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
        }),
        headers: {
            'Content-Type': 'application/json',
        },
        credentials: 'include',
    });

    const json_res = await res.json();

    const updated_nodes = json_res['success']['updated_nodes'];
    const removed_nodes = json_res['success']['removed_nodes'];

    return [updated_nodes, removed_nodes]
};


const updateLoop = async (graphManager, lens) => {
    try {
        console.info('Fetching updates');
        const [updated_nodes, removed_nodes] = await retrieveGraph(
            graphManager.graph, lens
        );

        console.log('updated_nodes ', updated_nodes);

        if (updated_nodes.length !== 0) {
            const update =  dgraphNodesToD3Format(updated_nodes);
            graphManager.updateGraph(update);
        }

        if (removed_nodes.length !== 0) {
            // graphManager.removeNodesAndLinks(removed_nodes);
        }
    } catch (e) {
        console.warn("Failed to fetch updates ", e)
    }

    setTimeout(async () => {
        await updateLoop(graphManager, lens);
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

    document.getElementById('LensHeader').innerText = `Lens ${lens}`;

    // console.log("Initializing graphManager with, ", initGraph);
    const graphManager = new GraphManager(
        {nodes: [], links: []}, '2d'
    );
    console.log("Starting update loop");
    await updateLoop(graphManager, lens);
});
