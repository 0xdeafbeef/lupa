package stress

sealed interface Screen {
    data object Loading : Screen

    data class Ready(val count: Int) : Screen
}

enum class Tone {
    Good,
    Bad,
}

data class TradeWatchItem(val ticker: String, val price: Double) {
    val priceText: String
        get() = "$" + price.toString()

    companion object Factory {
        val fallback = TradeWatchItem("TON", 0.0)

        fun fromTicker(ticker: String): TradeWatchItem = TradeWatchItem(ticker, 1.0)
    }

    fun label(prefix: String): String {
        val normalize = { value: String -> value.trim().uppercase() }
        return "$prefix:${normalize(ticker)}"
    }
}

object TradeWatchRegistry {
    val default = TradeWatchItem("TON", 1.0)

    companion object {
        val empty = TradeWatchItem("NONE", 0.0)
    }

    fun build(): TradeWatchItem = default
}

fun TradeWatchItem.tone(): Tone = if (price > 0) Tone.Good else Tone.Bad

val appName = "Trade"
