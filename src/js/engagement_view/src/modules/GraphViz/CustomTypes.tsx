
export type LoginProps = {
    loginSuccess: () => void,
}

export type RouteState = {
    curPage: string,
    lastCheckLoginCheck: number,
}

export type SetRouteState = (routeState: RouteState) => void; 

export type VizNode = { 
    name: number,
    risk: number,
    id: number,
    nodeType: string, 
    nodeLabel: string,
}

export type LinkType = {
    source: VizNode, 
    target: VizNode, 
    name: string,
}

export interface BaseNode {
    dgraph_type: string[], 
    uid: number, 
    node_type: string,
    node_key: string ,
    risk: number | undefined,
    analyzer_names: string | undefined,
    risks: Risk[] | undefined,
}

export interface Process extends BaseNode {
    process_id: number,
    created_timestamp: number, 
    terminate_time: number,
    image_name: string, 
    process_name: string,
    arguments: string,
    children: Process[],
    bin_file: File[],
    created_files: File,
    deleted_files: File,
    read_files: File[],
    wrote_files: File[],
    created_connections: ProcessOutboundConnection[],
    inbound_connections: ProcessInboundConnection[],
}

export interface Risk extends BaseNode {
    analyzer_name: string,
    risk_score: number,
}

export interface Asset extends BaseNode {
    hostname: string,
    asset_ip: IpAddress[],
    asset_processes: Process[],
    files_on_asset: File[],
}

export interface File extends BaseNode {
    file_name: string,
    file_path: string,
    file_extension: string,
    file_mime_type: string,
    file_size: number,
    file_version: string, 
    file_description: string,
    file_product: string,
    file_company: string, 
    file_directory: string,
    file_inode: number,
    file_hard_links: string, 
    signed: boolean,
    signed_status: string, 
    md5_hash: string,
    sha1_hash: string,
    sha256_hash: string,
}

export interface IpConnections extends BaseNode {
    src_ip_addr: string,
    src_port: string,
    dst_ip_addr: string,
    dst_port: string,
    created_timestamp: number,
    terminated_timestamp: number,
    last_seen_timestamp: number,
    inbound_ip_connection_to: IpAddress,
}

export interface IpAddress extends BaseNode {
    ip_address: string, 
    first_seen_timestamp: number, 
    last_seen_timestamp: number,
    ip_connections: IpConnections[],
}

export interface NetworkConnection extends BaseNode {
    src_ip_address: string, 
    src_port: string, 
    dst_ip_address: string, 
    dst_port: string, 
    created_timestamp: number, 
    terminated_timestamp: number, 
    last_seen_timestamp: number,
    inbound_network_connection_to: IpPort[]
}

export interface IpPort extends BaseNode {
    ip_address: string,
    protocol: string,
    port: number, 
    first_seen_timestamp: number, 
    last_seen_timestamp: number, 
    network_connections: NetworkConnection[],
}

export interface ProcessInboundConnection extends BaseNode {
    ip_address: string,
    protocol: string, 
    created_timestamp: number, 
    terminated_timestamp: number,
    last_seen_timestamp: number,
    port: number,
    bound_port: IpPort[],
    bound_ip: IpAddress[],
}

export interface ProcessOutboundConnection extends BaseNode {
    ip_address: string,
    protocol: string, 
    created_timestamp: number, 
    terminated_timestamp: number,
    last_seen_timestamp: number,
    port: number,
    connected_over: IpPort[],
    connected_to: IpPort[],
}

export type Dynamic = BaseNode;

export interface Lens extends BaseNode {
    lens_name: string,
    score: number,
}

export type LensScopeResponse = Lens;

// All Nodes except for Lesns / Risk 
export type Scope = Asset | Process | Dynamic | IpAddress | IpPort | NetworkConnection | ProcessInboundConnection | ProcessOutboundConnection 

export type Node = Scope | Lens | Risk


export type VizGraph = {
    links: LinkType[], 
    nodes: VizNode[],
}

export type UidType = {
    uid: number,
}

export type MergeLinkType = {
    source: UidType,
    label: string,
    target: UidType,
}

export type MergeGraphType = { 
    nodes: Node[],
    links: MergeLinkType[],
}
