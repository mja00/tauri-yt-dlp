import { Dialog as SheetPrimitive } from 'bits-ui';



const Root = SheetPrimitive.Root;
const Close = SheetPrimitive.Close;
const Trigger = SheetPrimitive.Trigger;
const Portal = SheetPrimitive.Portal;

export {
	Root,
	Close,
	Trigger,
	Portal,



	//
	Root as Sheet,
	Close as SheetClose,
	Trigger as SheetTrigger,
	Portal as SheetPortal,



};

export { default as Overlay, default as SheetOverlay } from './sheet-overlay.svelte';
export { default as Content, default as SheetContent } from './sheet-content.svelte';

export { default as Header, default as SheetHeader } from './sheet-header.svelte';
export { default as Footer, default as SheetFooter } from './sheet-footer.svelte';

export { default as Title, default as SheetTitle } from './sheet-title.svelte';

export { default as Description, default as SheetDescription } from './sheet-description.svelte';
