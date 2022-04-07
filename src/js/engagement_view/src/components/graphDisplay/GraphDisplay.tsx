import React, {
    useEffect,
    useState,
    useMemo,
    useCallback,
    useRef,
} from "react";
import ForceGraph2D from "react-force-graph-2d";
import { nodeFillColor, riskOutline } from "./graphVizualization/nodeColoring";
import {
    calcLinkParticleWidth,
    calcLinkColor,
    calcLinkDirectionalArrowRelPos,
} from "./graphVizualization/linkCalcs";
import { nodeSize } from "./graphVizualization/nodeCalcs";
import { getLinkLabel } from "./graphLayout/labels";
import { updateGraph } from "./graphUpdates/updateGraph";
import { Link, VizNode, VizGraph } from "../../types/CustomTypes";
import {
    GraphState,
    GraphDisplayState,
    GraphDisplayProps,
} from "../../types/GraphDisplayTypes";

import { colors } from "./graphVizualization/graphColors";

type ClickedNodeState = VizNode | null;

const defaultGraphDisplayState = (
    lensName: string | null
): GraphDisplayState => {
    return {
        graphData: { nodes: [], links: [], index: {} },
        curLensName: lensName,
    };
};

const defaultClickedState = (): ClickedNodeState => {
    return null;
};

const GraphDisplay = ({ lensName, setCurNode }: GraphDisplayProps) => {
    const fgRef: any = useRef(); // fix graph to canvas
    const [state, setState] = useState(defaultGraphDisplayState(lensName));

    async function updateGraphAndSetState() {
        if (lensName) {
            await updateGraph(lensName, state as GraphState, setState); // state is safe cast, check that lens name is not null
        }
    }

    // TODO is there a way to updateGraphAndSetState immediately on click?

    useEffect(() => {
        // Set the initial state immediately
        updateGraphAndSetState();

        // refresh every 10 seconds
        const interval = setInterval(updateGraphAndSetState, 10000);
        return () => clearInterval(interval);
    }, [lensName, state, setState, updateGraphAndSetState]);

    const data = useMemo(() => {
        const graphData = state.graphData;
        return graphData;
    }, [state]);

    const [clickedNode, setClickedNode] = useState(defaultClickedState());
    const [highlightNodes, setHighlightNodes] = useState(new Set());
    const [highlightLinks, setHighlightLinks] = useState(new Set());
    const [hoverNode, setHoverNode] = useState(null);

    const updateHighlight = useCallback(() => {
        setHighlightNodes(highlightNodes);
        setHighlightLinks(highlightLinks);
    }, [highlightNodes, highlightLinks]);

    const nodeClick = useCallback(
        (_node, ctx) => {
            const node = _node as any;
            const links = node.links;
            const neighbors = node.neighbors;

            // remove neighbors and links for node detail table iteration (react can only iterate through arrays)
            delete node.links;
            delete node.neighbors;

            setCurNode(node);
            setClickedNode(node || null);

            // re-add neighbors for highlighting links
            node.links = links;
            node.neighbors = neighbors;
        },
        [setCurNode, setClickedNode]
    );

    const nodeHover = useCallback(
        (node, ctx) => {
            highlightNodes.clear();
            highlightLinks.clear();

            if (node) {
                const _node = node as any;
                highlightNodes.add(_node);

                if (!_node.neighbors) {
                    return;
                }

                _node.neighbors.forEach((neighbor: VizNode) => {
                    highlightNodes.add(neighbor);
                });
                _node.links.forEach((link: Link) => {
                    highlightLinks.add(link);
                });
            }

            setHoverNode((node as any) || null);
            updateHighlight();
        },
        [setHoverNode, updateHighlight, highlightLinks, highlightNodes]
    );

    const nodeStyling = useCallback(
        (node, ctx) => {
            node.fx = node.x;
            node.fy = node.y;
            ctx.save();

            const NODE_R = nodeSize(node, data);

            // Node Border Styling
            ctx.beginPath();
            ctx.arc(node.x, node.y, NODE_R * 1.4, 0, 2 * Math.PI, false);
            ctx.fillStyle =
                node === hoverNode
                    ? colors.hoverNodeFill
                    : riskOutline(node.risk_score);
            ctx.fill();
            ctx.save();

            // Node Fill Styling
            ctx.beginPath();
            ctx.arc(node.x, node.y, NODE_R * 1.2, 0, 2 * Math.PI, false);
            ctx.fillStyle =
                node === clickedNode
                    ? colors.clickedNode
                    : nodeFillColor(node.dgraph_type[0]);
            ctx.fill();
            ctx.save();

            // Node Label Styling
            const label = node.nodeLabel;

            const fontSize = Math.min(
                98,
                NODE_R / ctx.measureText(label).width
            );
            ctx.font = `${fontSize + 5}px Roboto`;

            const textWidth = ctx.measureText(label).width;
            const labelBkgdDimensions = [textWidth, fontSize].map(
                (n) => n + fontSize * 0.2
            );

            ctx.fillStyle = colors.nodeLabelFill;
            ctx.fillRect(
                node.x - labelBkgdDimensions[0] / 2, // x coordinate
                node.y - labelBkgdDimensions[1] - 2.75, // y coordinate
                labelBkgdDimensions[0] + 1.25, // rectangle width
                labelBkgdDimensions[1] + 5.5 // rectangle height
            );
            ctx.textAlign = "center";
            ctx.textBaseline = "middle";
            ctx.fillStyle = colors.nodeLabelTxt;
            ctx.fillText(label, node.x, node.y);
            ctx.save();
        },
        [data, clickedNode, hoverNode]
    );

    const linkStyling = (link: any, ctx: any) => {
        const MAX_FONT_SIZE = 8;
        const LABEL_NODE_MARGIN = 12;
        const start = link.source;
        const end = link.target;

        link.color = calcLinkColor(link, data);

        // Ignore unbounded links
        if (typeof start !== "object" || typeof end !== "object") return;

        // Edge label positioning calculations
        const textPos = {
            x: start.x + (end.x - start.x) / 2,
            y: start.y + (end.y - start.y) / 2,
        };

        const relLink = { x: end.x - start.x, y: end.y - start.y };
        const maxTextLength =
            Math.sqrt(Math.pow(relLink.x, 2) + Math.pow(relLink.y, 2)) -
            LABEL_NODE_MARGIN * 8;

        let textAngle = Math.atan2(relLink.y, relLink.x);

        // Maintain label vertical orientation for legibility
        if (textAngle > Math.PI / 2) textAngle = -(Math.PI - textAngle);
        if (textAngle < -Math.PI / 2) textAngle = -(-Math.PI - textAngle);

        const label = getLinkLabel(link.name);

        // Estimate fontSize to fit in link length
        ctx.font = "50px Roboto";
        const fontSize = Math.min(
            MAX_FONT_SIZE,
            maxTextLength / ctx.measureText(label).width
        );
        ctx.font = `${fontSize + 5}px Roboto`;

        // Draw text label
        ctx.save();
        ctx.translate(textPos.x, textPos.y);
        ctx.rotate(textAngle);
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(label, 0.75, 3); //Content, left/right, top/bottom
        ctx.restore();
    };

    return (
        <ForceGraph2D
            graphData={data}
            ref={fgRef} // fix graph to canvas
            nodeLabel={"nodeType"} // tooltip on hover, actual label is in nodeCanvasObject
            nodeCanvasObject={nodeStyling}
            nodeCanvasObjectMode={() => "after"}
            onNodeHover={nodeHover}
            onNodeClick={nodeClick}
            onNodeDragEnd={(node) => {
                node.fx = node.x;
                node.fy = node.y;
            }}
            linkColor={(link) =>
                highlightLinks.has(link)
                    ? colors.highlightLink
                    : calcLinkColor(link as Link, data as VizGraph)
            }
            linkWidth={(link) => (highlightLinks.has(link) ? 5 : 4)}
            linkDirectionalArrowLength={10}
            linkDirectionalArrowRelPos={(link) => {
                const _link = link as any;
                return calcLinkDirectionalArrowRelPos(_link, data);
            }}
            linkDirectionalParticleSpeed={0.005}
            linkDirectionalParticleColor={(link) => colors.linkDirParticle}
            linkDirectionalParticles={1}
            linkDirectionalParticleWidth={(link) =>
                highlightLinks.has(link)
                    ? 4
                    : calcLinkParticleWidth(link as Link, data as VizGraph) + 1
            }
            linkCanvasObjectMode={() => "after"}
            linkCanvasObject={linkStyling}
            onLinkHover={(link) => {
                highlightNodes.clear();
                highlightLinks.clear();

                if (link) {
                    highlightLinks.add(link);
                    highlightNodes.add(link.source);
                    highlightNodes.add(link.target);
                }
            }}
            warmupTicks={100}
            cooldownTicks={100}
        />
    );
};

export default GraphDisplay;
