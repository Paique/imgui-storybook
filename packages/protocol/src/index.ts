/**
 * Wire protocol between the web client and the imgui-storybook host.
 * Mirrors java-host `host/net/Protocol.java`.
 */

export type Theme = 'light' | 'dark';
export type Backdrop = 'neutral-dark' | 'neutral-light' | 'checker';
export type CanvasMode = 'windowed' | 'inline';

export interface ViewOptions {
  theme: Theme;
  scale: number;
  backdrop: Backdrop;
  canvasMode: CanvasMode;
}

export interface HelloMsg {
  type: 'hello';
  host: string;
  hostVersion: string;
  imguiJavaVersion: string;
  dearImgui: string;
  width: number;
  height: number;
  fbWidth: number;
  fbHeight: number;
}

export interface SelectedMsg {
  type: 'selected';
  storyId: string;
  args: Record<string, unknown>;
  view: ViewOptions;
}

export interface ViewMsg {
  type: 'view';
  view: ViewOptions;
}

export interface FrameMsg {
  type: 'frame';
  seq: number;
  width: number;
  height: number;
  mime: string;
  /** base64-encoded JPEG */
  data: string;
}

export interface ActionMsg {
  type: 'action';
  name: string;
  t: number;
}

export interface ErrorMsg {
  type: 'error';
  message: string;
}

export type ServerMessage =
  | HelloMsg
  | SelectedMsg
  | ViewMsg
  | FrameMsg
  | ActionMsg
  | ErrorMsg;

export type InputKind =
  | 'mouseMove'
  | 'mouseDown'
  | 'mouseUp'
  | 'mouseLeave'
  | 'wheel'
  | 'keyDown'
  | 'keyUp'
  | 'text';

export interface InputEventMsg {
  type: 'input';
  kind: InputKind;
  x?: number;
  y?: number;
  button?: number;
  dx?: number;
  dy?: number;
  /** browser KeyboardEvent.code */
  key?: string;
  text?: string;
}

export type ClientMessage =
  | { type: 'select'; storyId: string }
  | { type: 'setArgs'; args: Record<string, unknown> }
  | { type: 'setView'; view: Partial<ViewOptions> }
  | InputEventMsg
  | { type: 'ping' };

/** Arg metadata as served by `host --list --json`. */
export type ArgKind = 'STRING' | 'BOOLEAN' | 'INT' | 'FLOAT' | 'ENUM' | 'COLOR';

export interface ArgInfo {
  name: string;
  type: ArgKind;
  default: string | number | boolean;
  options?: string[];
}

export interface StoryInfo {
  id: string;
  title: string;
  description: string;
  storyClass: string;
  args: ArgInfo[];
  presets: Record<string, Record<string, string | number | boolean>>;
}

export interface Catalog {
  hostVersion: string;
  imguiJavaVersion: string;
  dearImgui: string;
  stories: StoryInfo[];
}

/** One captured image as described by captures.json. */
export interface CaptureEntry {
  storyId: string;
  title: string;
  description: string;
  storyClass: string;
  preset: string;
  theme: 'light' | 'dark';
  scale: number;
  /** path relative to the captures root, e.g. imgui/basics-button/default-dark.png */
  file: string;
  width: number;
  height: number;
  args: Record<string, string | number | boolean>;
}

export interface CapturesManifest {
  generatedAt: string;
  hostVersion: string;
  imguiJavaVersion: string;
  dearImgui: string;
  width: number;
  height: number;
  entries: CaptureEntry[];
}
