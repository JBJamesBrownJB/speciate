When population is high 100k and latency is close to max 40ms or so we get jerky movement of crits.

It should be fine as latency is always below 50ms (unless our sampling is hiding breaching this though we don't get warnings of skipped / caught up frames in console).

It is even worse whith pub const PERCEPTION_SKIP_TICKS above zero.

Again, it appears that total_tick is always under 50 so there shouldn't be any jitter / jerky movement still.