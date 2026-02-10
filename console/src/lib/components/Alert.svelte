<script lang="ts">
	interface Props {
		type: 'success' | 'error' | 'warning' | 'info';
		message: string;
	}

	let { type, message }: Props = $props();
</script>

<div class="alert alert-{type}">
	<span>{message}</span>
</div>
