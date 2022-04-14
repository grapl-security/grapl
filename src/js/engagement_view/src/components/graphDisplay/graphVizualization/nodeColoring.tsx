export const nodeFillColor = (nodeType: string) => {
    switch (nodeType) {
        case "Asset":
            return "rgba(251, 154, 153, .8)"; //#FB9A99
        case "File":
            return "rgba(236, 189, 169, .8)"; //#ECBDA9
        case "IpAddress":
            return "rgba(253, 191, 111, .8)"; //#6CF4AB
        case "IpConnections":
            return "rgba(255, 241, 150, .8)"; //#FFF196
        case "IpPort":
            return "rgba(178, 223, 138, .8)"; // #B2DF8A
        case "NetworkConnection":
            return "rgba(166, 206, 227, .8)"; //#A6CEE3
        case "Process":
            return "rgba(31, 120, 180, .8)"; // #1F78B4
        case "ProcessInboundConnection":
            return "rgba(202, 178, 214, .8)"; //#CAB2D6
        case "ProcessOutboundConnection":
            return "rgba(106, 61, 154, .8)"; //#6A3D9A
        case "Risk":
            return "rgba(238, 238, 238, .8)"; //#EEEEEE
        default:
            return "#42C6FF";
    }
};

export const riskOutline = (risk: number) => {
    if (risk >= 0 && risk <= 25) {
        return "#7FE49F";
    }
    if (risk >= 26 && risk <= 50) {
        return "#13A5E3";
    }
    if (risk >= 51 && risk <= 75) {
        return "#FFD773";
    }
    if (risk >= 76 && risk <= 100) {
        return "#DA634F";
    }
};
