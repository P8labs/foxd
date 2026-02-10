<script lang="ts">
	interface Props {
		title: string;
		value: string | number;
		subtitle?: string;
	}

	let { title, value, subtitle }: Props = $props();
</script>

<div class="stat-card">
	<div class="stat-title">{title}</div>
	<div class="stat-value">{value}</div>
	{#if subtitle}
		<div class="stat-subtitle">{subtitle}</div>
	{/if}
</div>
