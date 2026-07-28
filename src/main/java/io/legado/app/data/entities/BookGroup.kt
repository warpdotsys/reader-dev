package io.legado.app.data.entities




//@Parcelize
//@Entity(tableName = "book_groups")
data class BookGroup(
//        @PrimaryKey
        var groupId: Long = 0L,
        var groupName: String = "",
        var order: Int = 0,
        var show: Boolean = true,
        var cover: String? = null
)
