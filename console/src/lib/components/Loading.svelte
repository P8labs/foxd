<script lang="ts">
	interface Props {
		message?: string;
	}

	let { message = 'Loading...' }: Props = $props();
</script>

<div class="loading">
	<div class="spinner"></div>
	<p class="text-muted">{message}</p>
</div>
