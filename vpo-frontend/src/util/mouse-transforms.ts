import { PanZoom } from "panzoom";

export function transformMouse(panzoom: PanZoom, mouseX: number, mouseY: number): [number, number] {
    const transform = panzoom.getTransform();

    const transformedX = (mouseX - transform.x) / transform.scale;
    const transformedY = (mouseY - transform.y) / transform.scale;

    return [transformedX, transformedY];
}
