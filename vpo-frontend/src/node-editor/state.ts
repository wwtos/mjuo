import { BehaviorSubject } from "rxjs";
import { SocketRegistry } from "../node-engine/socket_registry";

export const socketRegistry: BehaviorSubject<SocketRegistry> = new BehaviorSubject(new SocketRegistry());
