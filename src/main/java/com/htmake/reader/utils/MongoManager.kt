package com.htmake.reader.utils

import com.mongodb.MongoException
import com.mongodb.MongoClientSettings
import com.mongodb.client.MongoClient
import com.mongodb.client.MongoClients
import com.mongodb.client.MongoCollection
import com.mongodb.client.MongoDatabase
import org.bson.codecs.configuration.CodecRegistries
import org.bson.codecs.configuration.CodecProvider
import org.bson.codecs.pojo.PojoCodecProvider
import com.htmake.reader.entity.MongoFile

object MongoManager {
    private var mongoClient: MongoClient? = null

    fun isInit(): Boolean {
        return mongoClient != null
    }

    fun connect(uri: String) {
        try {
            val client = MongoClients.create(uri)
            mongoClient = client
        } catch (e: MongoException) {
            logger.info("mongodb 连接失败，请检查链接({})是否正确", uri)
            e.printStackTrace()
        }
    }

    fun db(name: String): MongoDatabase? {
        if (!isInit()) {
            return null
        }
        val pojoCodecProvider = PojoCodecProvider.builder().automatic(true).build()
        val codecRegistry = CodecRegistries.fromRegistries(
            MongoClientSettings.getDefaultCodecRegistry(),
            CodecRegistries.fromProviders(pojoCodecProvider as CodecProvider)
        )
        return mongoClient!!.getDatabase(name).withCodecRegistry(codecRegistry)
    }

    fun fileStorage(dbName: String, collection: String): MongoCollection<MongoFile>? {
        val database = db(dbName) ?: return null
        return database.getCollection(collection, MongoFile::class.java)
    }
}
