import { LinkNode } from "./savefile-model";

export interface Coordinates {
  x: number;
  y: number;
}

export function deserializeCoordinates(node: LinkNode, mapSizeX: number): Coordinates {
  const logMapX = Math.log2(mapSizeX);
  return { x: node.xy & (mapSizeX - 1), y: node.xy >> logMapX };
}

export function transposeCoordinates(mapSizeX: number, coords: Coordinates, mapSizeY: number) {
  return { x: mapSizeX - coords.x, y: mapSizeY - coords.y };
}