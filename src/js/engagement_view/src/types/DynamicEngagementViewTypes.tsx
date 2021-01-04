import {Node, Lens} from './CustomTypes'

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

export type EngagementViewContentProps = {
    setLens: (lens: string) => void, 
    curNode: Node | null
}