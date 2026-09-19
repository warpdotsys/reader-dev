package com.htmake.reader.utils

import com.htmake.reader.config.AppConfig
import com.htmake.reader.entity.License
import io.legado.app.utils.EncoderUtils
import java.security.KeyFactory
import java.security.spec.X509EncodedKeySpec
import java.util.Base64

private const val LEGACY_LICENSE_PUBLIC_KEY =
    "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAj0G3qEPjVTvVd7pXFUVYZFHT8KaoG4onc5rLUKqFQ2DCh/5hFK9t2nKh2XB+C2Jp/GSK2ONwD7ceXenmA6uvr90uCK/gp6j62XFVRvc8sIm0d/bGbzZFJRk3HKtxEckBmASduPObY691DVVixxNtUrSJktx/TZaB42pUQk4j+7FuOVNNPra44hDdnyGhmYBBf2B4kjXVMjL+0NCblFIN1+qjmcol44k6NFKFF54q05bjR3CRyYdAnNTCOyt9va0oB6lDlKHplSZmAOH9JGMUki/HDJbABESXMnyIpux27w9SQ8aJStYttnJWHALO1hiFJsxbz5KUkldH6Ny1p/2W5QIDAQAB"

@Volatile
private var licenseOnlineValid = true

fun setLicenseValid(value: Boolean) {
    licenseOnlineValid = value
}

fun getInstalledLicense(ignoreOnlineInvalid: Boolean = false): License {
    val encrypted = getStorage("data", "license", ext = ".key")
    if (encrypted.isNullOrBlank() || (!ignoreOnlineInvalid && !licenseOnlineValid)) return License()
    return decryptToLicense(encrypted)?.takeIf { it.verified } ?: License()
}

fun decryptToLicense(encrypted: String): License? =
    decryptLicenseData(encrypted)?.let { runCatching { it.toMap().toDataClass<License>() }.getOrNull() }

fun decryptLicenseData(content: String): String? {
    if (content.isBlank()) return null
    val configured = SpringContextUtils.getBean("appConfig", AppConfig::class.java)?.licensePublicKey.orEmpty().trim()
    return sequenceOf(configured, LEGACY_LICENSE_PUBLIC_KEY)
        .filter { it.isNotEmpty() }
        .distinct()
        .mapNotNull { publicKeyText ->
            runCatching {
                val publicKey = KeyFactory.getInstance("RSA").generatePublic(
                    X509EncodedKeySpec(Base64.getDecoder().decode(publicKeyText))
                )
                EncoderUtils.decryptSegmentByPublicKey(content, publicKey)
            }.getOrNull()
        }
        .firstOrNull()
}
