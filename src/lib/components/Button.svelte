<script lang="ts">
    type Variant = 'primary' | 'success' | 'danger';
    type Size = 'sm' | 'md' | 'lg';

    export let onClick: () => void = () => {};
    export let disabled: boolean = false;
    export let variant: Variant = 'primary';
    export let size: Size = 'md';
    export let fullWidth: boolean = false;

    let classes: string = '';

    $: {
    	const baseClasses = 'border-none rounded-lg font-medium cursor-pointer transition-colors duration-200 disabled:cursor-not-allowed';
    	const variantClasses: Record<Variant, string> = {
    		primary: 'bg-primary text-white hover:bg-primary-dark disabled:bg-[#95a5a6]',
    		success: 'bg-[#27ae60] text-white hover:bg-[#229954] disabled:bg-[#95a5a6]',
    		danger: 'bg-[#e74c3c] text-white hover:bg-[#c0392b] disabled:bg-[#95a5a6]',
    	};
    	const sizeClasses: Record<Size, string> = {
    		sm: 'px-5 py-2.5 text-sm',
    		md: 'px-5 py-2.5 text-sm',
    		lg: 'py-3.5 px-5 text-base font-semibold',
    	};
    	const widthClass = fullWidth ? 'flex-1' : '';

    	classes = `${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${widthClass}`;
    }
</script>

<button class={classes} {disabled} on:click={onClick}>
    <slot />
</button>
