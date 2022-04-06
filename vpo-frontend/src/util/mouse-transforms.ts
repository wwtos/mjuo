import { PanZoom } from "panzoom";

export function transformMouse(panzoom: PanZoom, mouseX: number, mouseY: number): [number, number] {
    const transform = panzoom.getTransform();

    const transformedX = (mouseX - transform.x) / transform.scale;
    const transformedY = (mouseY - transform.y) / transform.scale;

    return [transformedX, transformedY];
}

export function transformMouseRelativeToEditor(editor: HTMLDivElement, panzoom: PanZoom, mouseX: number, mouseY: number) {
    let boundingRect = editor.getBoundingClientRect();

    let relativeX = mouseX - boundingRect.x;
    let relativeY = mouseY - boundingRect.y;

    return transformMouse(panzoom, relativeX, relativeY);
}