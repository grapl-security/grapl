import {Node} from '../../graphViz/utils/GraphVizCustomTypes'

export type SelectLensProps = {
    lens: string,
    score: number,
    uid: number,
    lens_type: string,
    setLens: (lens: string) => void,
}

export type NodeDetailsProps = {
    node: Node
}

export type ToggleNodeTableProps = {
    curNode: Node | null
}