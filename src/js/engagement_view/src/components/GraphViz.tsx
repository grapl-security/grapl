// @ts-nocheck
import React, { useEffect, useRef } from 'react';
import { ForceGraph2D } from 'react-force-graph';
import * as d3 from "d3";
import { BKDRHash, riskColor, calcNodeRgb , calcLinkColor} from "../modules/GraphViz/graphColoring/coloring.tsx";
import { retrieveGraph } from '../modules/GraphViz/graphQL/expandScope.tsx';
import { mapLabel } from '../modules/GraphViz/graph/labels.tsx';
import { nodeSize } from '../modules/GraphViz/calculations/node/nodeCalcs.tsx'
import { calcLinkDirectionalArrowRelPos, calcLinkParticleWidth  } from '../modules/GraphViz/calculations/link/linkCalcs.tsx'
import {mergeGraphs} from '../modules/GraphViz/graph/mergeGraphs.tsx'
import {graphQLAdjacencyMatrix} from '../modules/GraphViz/graphQL/graphQLAdjacencyMatrix.tsx'
import { Node, LinkType, GraphType, ColorHashOptions } from "../modules/GraphViz/CustomTypes"

type ColorHashOptions = {
    lightness: number,
    saturation: number,
    hue: number,
    hash: BKDRHash,
}

/**
 * Convert HSL to RGB
 *
 * @see {@link http://zh.wikipedia.org/wiki/HSL和HSV色彩空间} for further information.
 * @param {Number} H Hue ∈ [0, 360)
 * @param {Number} S Saturation ∈ [0, 1]
 * @param {Number} L Lightness ∈ [0, 1]
 * @returns {Array} R, G, B ∈ [0, 255]
 */
const HSL2RGB = (H: number, S: number, L: number) => {
    H /= 360;

    const q = L < 0.5 ? L * (1 + S) : L + S - L * S;
    const p = 2 * L - q;

    return [H + 1 / 3, H, H - 1 / 3].map((color) => {
        if (color < 0) {
            color++;
        }
        if (color > 1) {
            color--;
        }
        if (color < 1 / 6) {
            color = p + (q - p) * 6 * color;
        } else if (color < 0.5) {
            color = q;
        } else if (color < 2 / 3) {
            color = p + (q - p) * 6 * (2 / 3 - color);
        } else {
            color = p;
        }
        return Math.round(color * 255);
    });
};

const isArray = (o: Object) => {
    return Object.prototype.toString.call(o) === '[object Array]';
};

/**
 * Color Hash Class
 *
 * @class
 */

export class ColorHash {
    constructor(options: ColorHashOptions | undefined) {
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
        this.hueRanges = options.hue.map(function (range: number) {
            return {
                min: typeof range.min === 'undefined' ? 0 : range.min,
                max: typeof range.max === 'undefined' ? 360 : range.max
            };
        });
    
        this.hash = options.hash || BKDRHash;
    } 
    /**
     * Returns the hash in [h, s, l].
     * Note that H ∈ [0, 360); S ∈ [0, 1]; L ∈ [0, 1];
     *
     * @param {String} str string to hash
     * @returns {Array} [h, s, l]
     */
    hsl = (str: string) => {
        let H, S, L;
        let hash = this.hash(str);
    
        if (this.hueRanges.length) {
            const range = this.hueRanges[hash % this.hueRanges.length];
            const hueResolution = 727; // note that 727 is a prime
            H = ((hash / this.hueRanges.length) % hueResolution) * (range.max - range.min) / hueResolution + range.min;
        } else {
            H = hash % 359; // note that 359 is a prime
        }
        hash = parseInt(hash / 360 as any);
        S = this.S[hash % this.S.length];
        hash = parseInt(hash / this.S.length as any);
        L = this.L[hash % this.L.length];
    
        return [H, S, L];
    }

    /**
     * Returns the hash in [r, g, b].
     * Note that R, G, B ∈ [0, 255]
     *
     * @param {String} str string to hash
     * @returns {Array} [r, g, b]
     */
    rgb = (str: string) => {
        const hsl = this.hsl(str);
        return HSL2RGB.apply(this, hsl);
    };
}


export const mapNodeProps = (node: Node, f: (string) => void) => {
    for (const prop in node) {
        if (Object.prototype.hasOwnProperty.call(node, prop)) {
            if (Array.isArray(node[prop])) {
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

type GraphState = {
    curLensName: LensType[], 
    graphData: GraphType[]
}

const updateGraph = async (
    lensName: string, 
    state: GraphState, 
    setState: (state: GraphState) => void,
) => {
    if (!lensName) {
        return;
    }

    await retrieveGraph(lensName)
        .then(async (scope) => {
            const update = graphQLAdjacencyMatrix(scope);
            console.log('update', update);
            const mergeUpdate = mergeGraphs(state.graphData, update);
            if (mergeUpdate !== null) {
                if (state.curLensName === lensName) {
                    setState({
                        ...state,
                        curLensName: lensName,
                        graphData: mergeUpdate,
                    })
                } else {
                    setState({
                        ...state,
                        curLensName: lensName,
                        graphData: update,
                    })
                }
            }
        })
        .catch((e) => console.error("Failed to retrieveGraph ", e))
}

type GraphDisplayProps = {
    lensName: string | null,
    setCurNode: (string) => void,
}

type GraphDisplayState = {
    graphData: AdjacencyMatrix,
    curLensName: string | null,
}

const defaultGraphDisplayState = (lensName: string): GraphDisplayState => {
    return {
        graphData: {nodes: [], links: []},
        curLensName: lensName,
    }
}

const GraphDisplay = ({lensName, setCurNode}: GraphDisplayProps) => {
    const [state, setState] = React.useState(defaultGraphDisplayState(lensName));
    const forceRef = useRef(null);

    useEffect(() => {
        // console.log("useEffect - setting forceRef state");
        forceRef.current.d3Force("link", d3.forceLink());
        forceRef.current.d3Force('collide', d3.forceCollide(22));   
        forceRef.current.d3Force("charge", d3.forceManyBody());
        forceRef.current.d3Force('box', () => {
            const N = 100;
            // console.log(Graph.width(), Graph.height())
            const SQUARE_HALF_SIDE = 20 * N * 0.5;
            state.graphData.nodes.forEach(node => {
                const x = node.x || 0, y = node.y || 0;
                // bounce on box walls
                if (Math.abs(x) > SQUARE_HALF_SIDE) {
                    node.vx *= -1;
                }
                if (Math.abs(y) > SQUARE_HALF_SIDE) {
                    node.vy *= -1;
                }
            });
        });
    }, [state])


    useEffect(() => {
        updateGraph(lensName, state, setState);
        const interval = setInterval(async () => {
            if (lensName) {
                await updateGraph(lensName, state, setState);
            }
        }, 1000);
        return () => clearInterval(interval);
    }, [lensName, state]);

    const graphData = state.graphData;

    const colorHash = new ColorHash({});

    // #TODO: ADD ZOOM HANDLERS FOR MAX ZOOM IN/OUT

    return (
        <>
            <ForceGraph2D
                graphData={graphData}
                nodeLabel={(node: Node) => node.nodeLabel}
                enableNodeDrag={true}
                linkDirectionalParticles={1}
                linkDirectionalParticleWidth={(link) => {
                    return calcLinkParticleWidth(link, graphData);
                }}
                linkDirectionalParticleColor={(link) => {
                    return calcLinkColor(link, graphData)
                }}
                linkDirectionalParticleSpeed={0.005}
                onNodeClick={
                    (node: Node, event: string) => {
                        setCurNode(node);
                    }
                }
                linkDirectionalArrowLength={8}
                linkWidth={4}
                linkDirectionalArrowRelPos={(link => {
                    return calcLinkDirectionalArrowRelPos(link, graphData);
                })}
                linkCanvasObjectMode={(() => 'after')}
                linkCanvasObject={((link: LinkType, ctx: any) => {
                    const MAX_FONT_SIZE = 8;
                    const LABEL_NODE_MARGIN = 8 * 1.5;
                    const start = link.source;
                    const end = link.target;
                    // ignore unbound links
                    link.color = calcLinkColor(link, graphData);

                    if (typeof start !== 'object' || typeof end !== 'object') return;
                    // calculate label positioning
                    const textPos = Object.assign(
                        ...['x', 'y'].map((c: any) => (
                            {
                                [c]: start[c] + (end[c] - start[c]) / 2 // calc middle point
                            }
                        )) as any
                    );

                    const relLink = {x: end.x - start.x, y: end.y - start.y};

                    const maxTextLength = Math.sqrt(Math.pow(relLink.x, 2) + Math.pow(relLink.y, 2)) - LABEL_NODE_MARGIN * 8;

                    let textAngle = Math.atan2(relLink.y, relLink.x);
                    // maintain label vertical orientation for legibility
                    if (textAngle > Math.PI / 2) textAngle = -(Math.PI - textAngle);
                    if (textAngle < -Math.PI / 2) textAngle = -(-Math.PI - textAngle);

                    const label = mapLabel(link.label);
                    // estimate fontSize to fit in link length
                    ctx.font = '50px Arial';
                    const fontSize = Math.min(MAX_FONT_SIZE, maxTextLength / ctx.measureText(label).width);
                    ctx.font = `${fontSize + 5}px Arial`;

                    let textWidth = ctx.measureText(label).width;

                    textWidth += Math.round(textWidth * 0.25);

                    const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding
                    // draw text label (with background rect)
                    ctx.save();
                    ctx.translate(textPos.x, textPos.y);
                    ctx.rotate(textAngle);
                    ctx.fillStyle = 'rgb(115,222,255,1)';
                    ctx.fillRect(-bckgDimensions[0] / 2, -bckgDimensions[1] / 2, ...bckgDimensions);
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.fillStyle = 'white';
                    //content, left/right, top/bottom
                    ctx.fillText(label, .75, 3);
                    ctx.restore();
                })}
                nodeCanvasObject={((node: Node, ctx: any, globalScale: any) => {
                    // add ring just for highlighted nodes

                    const NODE_R = nodeSize(node, graphData);
                    ctx.save();

                    // Risk outline color
                    ctx.beginPath();
                    ctx.arc(node.x, node.y, NODE_R * 1.3, 0, 2 * Math.PI, false);
                    ctx.fillStyle = riskColor(node, graphData, colorHash);
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

                    const fontSize = 15 / globalScale;

                    ctx.font = `${fontSize}px Arial`;


                    const textWidth = ctx.measureText(label).width;

                    const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding
                    // node label color
                    ctx.fillStyle = 'rgba(48, 48, 48, 0.8)';
                    ctx.fillRect(node.x - bckgDimensions[0] / 2, node.y - bckgDimensions[1] / 2, ...bckgDimensions);
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.fillStyle = 'white';
                    ctx.fillText(label, node.x, node.y);

                })}
                ref={forceRef}
            />
        </>
    )
}

export default GraphDisplay;
