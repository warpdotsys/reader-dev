package com.htmake.reader.utils

/** Chromium is only the opt-in compatibility baseline, not the bundled Camoufox runtime. */
internal object ChromiumLaunchPolicy {
    // A later --disable-features argument can replace Playwright's own switch.
    // Keep all defaults from the pinned Playwright 1.63.0, not just our additions.
    // ChromiumLaunchPolicyTest checks this against the actual bundled driver;
    // a driver upgrade must deliberately review the defaults again.
    internal val playwrightDisabledFeatures = listOf(
        "AvoidUnnecessaryBeforeUnloadCheckSync", "DestroyProfileOnBrowserClose",
        "DialMediaRouteProvider", "GlobalMediaControls", "HttpsUpgrades", "LensOverlay",
        "MediaRouter", "PaintHolding", "ThirdPartyStoragePartitioning",
        "BlockOriginHeaderModificationOnRedirect", "Translate", "AutoDeElevate",
        "OptimizationHints", "msForceBrowserSignIn", "msEdgeUpdateLaunchServicesPreferredVersion"
    )

    // These are browser-owned omnibox services, not page network features.
    // Disabling only preconnect still leaves AI-mode eligibility requests;
    // disabling only AIM still leaves default-search preconnects.
    internal val readerDisabledFeatures = listOf(
        "PreconnectToSearch", "PreconnectFromKeyedService", "AimEnabled"
    )

    fun disabledFeaturesArgument(): String =
        "--disable-features=" + (playwrightDisabledFeatures + readerDisabledFeatures).joinToString(",")
}
