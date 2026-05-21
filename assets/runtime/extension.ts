/**
 * Utah Browser extension runtime contract (reference for Wasm injection).
 * Production extensions are compiled to Wasm and loaded by the Rust wasmi kernel.
 */

export type UtahExtensionTrigger = 'DOM_LOADED' | 'CLICK' | 'NAVIGATION';

export interface UtahExtensionManifest {
  name: string;
  trigger: UtahExtensionTrigger;
  intent: string;
  payload: Uint8Array;
}

export class ExtensionRuntime {
  private sandbox: WebAssembly.Instance | null = null;

  async loadExtension(manifest: UtahExtensionManifest): Promise<void> {
    const module = await WebAssembly.instantiate(manifest.payload);
    this.sandbox = module.instance;
    console.log(`[UTAH_RUNTIME] Extension ${manifest.name} injected into kernel.`);
  }

  execute(action: string): void {
    const exp = this.sandbox?.exports as { on_event?: (a: string) => void };
    exp.on_event?.(action);
  }
}
