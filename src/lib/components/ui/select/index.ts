import { Select as SelectPrimitive } from 'bits-ui';



const Root = SelectPrimitive.Root;
const Group = SelectPrimitive.Group;

export {
	Root,
	Group,



	//
	Root as Select,
	Group as SelectGroup,



};

export { default as GroupHeading, default as SelectGroupHeading } from './select-group-heading.svelte';

export { default as Item, default as SelectItem } from './select-item.svelte';

export { default as Content, default as SelectContent } from './select-content.svelte';

export { default as Trigger, default as SelectTrigger } from './select-trigger.svelte';

export { default as Separator, default as SelectSeparator } from './select-separator.svelte';

export { default as ScrollDownButton, default as SelectScrollDownButton } from './select-scroll-down-button.svelte';

export { default as ScrollUpButton, default as SelectScrollUpButton } from './select-scroll-up-button.svelte';
