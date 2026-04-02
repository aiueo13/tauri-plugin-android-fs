package com.plugin.android_fs

import android.annotation.SuppressLint
import android.app.Activity
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.ShareCompat
import app.tauri.annotation.InvokeArg
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger

class AFNotification {

    companion object {
        private val notificationIdCounter = AtomicInteger(0)
        private const val CHANNEL_ID = "progress_notification_channel"

        @Volatile
        private var isNotifiedProgressChannel = false

        private val notifications = ConcurrentHashMap.newKeySet<Int>()

        @Volatile
        private var notificationQueueManager: NotificationQueueManager? = null

        @InvokeArg
        enum class ProgressNotificationIconType {
            Download,
            Upload,
            Save,
            App,
        }

        private fun getIcon(i: ProgressNotificationIconType, ctx: Context): Int {
            return when (i) {
                ProgressNotificationIconType.Download -> android.R.drawable.stat_sys_download_done
                ProgressNotificationIconType.Upload -> android.R.drawable.stat_sys_upload_done
                ProgressNotificationIconType.Save -> android.R.drawable.ic_menu_save
                ProgressNotificationIconType.App -> {
                    ctx.applicationInfo.icon
                        .takeIf { it != 0 }
                        ?: android.R.drawable.sym_def_app_icon
                }
            }
        }

        private fun nextNotificationId(): Int {
            return notificationIdCounter.incrementAndGet()
        }

        @Synchronized
        fun initProgressNotificationManager(scope: CoroutineScope, ctx: Context) {
            if (!isNotifiedProgressChannel) {
                if (Build.VERSION_CODES.O <= Build.VERSION.SDK_INT) {
                    val name = "Progress Notification"
                    val description = "Notifies the progress and completion"
                    val importance = NotificationManager.IMPORTANCE_LOW
                    val channel = NotificationChannel(CHANNEL_ID, name, importance)
                    channel.description = description

                    val notificationManager = ctx.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
                    notificationManager.createNotificationChannel(channel)
                }
                isNotifiedProgressChannel = true
            }

            if (notificationQueueManager == null) {
                notificationQueueManager = NotificationQueueManager(scope)
            }
        }

        @SuppressLint("MissingPermission")
        suspend fun startProgressNotification(
            iconType: ProgressNotificationIconType,
            title: String?,
            text: String?,
            subText: String?,
            progressMax: Int?,
            progress: Int?,
            ctx: Context,
        ): Int {

            val id = nextNotificationId()
            notifications.add(id)

            notificationQueueManager?.add(id, NotificationEventType.Progress) {
                val builder = NotificationCompat.Builder(ctx, CHANNEL_ID)
                    .setSmallIcon(getIcon(iconType, ctx))
                    .setContentTitle(title.takeIf { !title.isNullOrEmpty() } ?: ctx.getString(android.R.string.unknownName))
                    .setCategory(NotificationCompat.CATEGORY_PROGRESS)
                    .setOngoing(false)
                    .setOnlyAlertOnce(true)

                if (progressMax != null && progress != null) {
                    builder.setProgress(progressMax, progress, false)
                }
                else {
                    builder.setProgress(0, 0, true)
                }

                if (!text.isNullOrEmpty()) {
                    builder.setContentText(text)
                }
                if (!subText.isNullOrEmpty()) {
                    builder.setSubText(subText)
                }

                NotificationManagerCompat.from(ctx).notify(id, builder.build())
            }

            return id
        }

        @SuppressLint("MissingPermission")
        suspend fun updateProgressNotification(
            id: Int,
            iconType: ProgressNotificationIconType,
            title: String?,
            text: String?,
            subText: String?,
            progressMax: Int?,
            progress: Int?,
            ctx: Context,
        ) {

            if (!notifications.contains(id)) {
                return
            }

            notificationQueueManager?.add(id, NotificationEventType.Progress) {
                if (!notifications.contains(id)) return@add

                val builder = NotificationCompat.Builder(ctx, CHANNEL_ID)
                    .setSmallIcon(getIcon(iconType, ctx))
                    .setContentTitle(title.takeIf { !title.isNullOrEmpty() } ?: ctx.getString(android.R.string.unknownName))
                    .setCategory(NotificationCompat.CATEGORY_PROGRESS)
                    .setOngoing(false)
                    .setOnlyAlertOnce(true)

                if (progressMax != null && progress != null) {
                    builder.setProgress(progressMax, progress, false)
                }
                else {
                    builder.setProgress(0, 0, true)
                }

                if (!text.isNullOrEmpty()) {
                    builder.setContentText(text)
                }
                if (!subText.isNullOrEmpty()) {
                    builder.setSubText(subText)
                }

                if (!notifications.contains(id)) return@add
                NotificationManagerCompat.from(ctx).notify(id, builder.build())
            }
        }

        @SuppressLint("MissingPermission")
        suspend fun finishProgressNotification(
            id: Int,
            iconType: ProgressNotificationIconType,
            title: String?,
            text: String?,
            subText: String?,
            error: Boolean,
            shareSrc: AFUri?,
            ctx: Context,
        ) {

            notifications.remove(id)

            notificationQueueManager?.add(id, NotificationEventType.ProgressFinish) {
                val icon = when (error) {
                    true -> android.R.drawable.stat_notify_error
                    else -> getIcon(iconType, ctx)
                }

                val builder = NotificationCompat.Builder(ctx, CHANNEL_ID)
                    .setSmallIcon(icon)
                    .setContentTitle(title.takeIf { !title.isNullOrEmpty() } ?: ctx.getString(android.R.string.unknownName))
                    .setOnlyAlertOnce(true)
                    .setOngoing(false)
                    .setAutoCancel(true)

                if (shareSrc != null) {
                    try {
                        val uri = Uri.parse(shareSrc.uri)
                        if (uri.scheme == "content") {
                            val mimeType = AFUtils.getFileMimeType(shareSrc, ctx)
                            val intentChooser = ShareCompat.IntentBuilder(ctx)
                                .setType(mimeType)
                                .setStream(uri)
                                .createChooserIntent().apply {
                                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                                    if (ctx is Activity) {
                                        putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(ctx.componentName))
                                    }
                                }

                            builder.setContentIntent(PendingIntent.getActivity(
                                ctx,
                                id,
                                intentChooser,
                                PendingIntent.FLAG_CANCEL_CURRENT or PendingIntent.FLAG_IMMUTABLE
                            ))
                        }
                    }
                    catch (ignore: Exception) { }
                }

                if (!text.isNullOrEmpty()) {
                    builder.setContentText(text)
                }
                if (!subText.isNullOrEmpty()) {
                    builder.setSubText(subText)
                }

                if (error) {
                    builder.setCategory(NotificationCompat.CATEGORY_ERROR)
                }

                NotificationManagerCompat.from(ctx).notify(id, builder.build())
            }
        }

        suspend fun cancelNotification(id: Int, ctx: Context) {
            notifications.remove(id)
            notificationQueueManager?.cancel(id)
            NotificationManagerCompat.from(ctx).cancel(id)
        }

        suspend fun cancelAllNotifications(ctx: Context) {
            notifications.clear()
            notificationQueueManager?.cancelAll()
            NotificationManagerCompat.from(ctx).cancelAll()
        }
    }
}


private enum class NotificationEventType {
    Progress,
    ProgressFinish,
}

/**
 * Android ではパッケージ単位での通知の送信・更新にレート制限があるため、
 * 優先度と遅延付きの Queue で通知を処理する。
 * https://saket.me/android-7-nougat-rate-limiting-notifications/
 */
private class NotificationQueueManager(scope: CoroutineScope) {
    private val queue = Channel<Pair<Int, NotificationEventType>>(Channel.UNLIMITED)
    private val notifications = HashMap<Pair<Int, NotificationEventType>, () -> Unit>()
    private val lock = Mutex()

    init {
        scope.launch {
            for (key in queue) {
                try {
                    val notificationTask = lock.withLock {
                        val notificationId = key.first
                        val notificationType = key.second
                        val notificationTask = notifications.remove(key) ?: return@withLock null

                        // Progress イベントは優先度が低いので必要に応じてスキップする。
                        if (notificationType == NotificationEventType.Progress) {
                            // 同じ通知に対して Finish イベントがあればそれを使用する。
                            val finishKey = Pair(notificationId, NotificationEventType.ProgressFinish)
                            val finishNotificationTask = notifications.remove(finishKey)
                            if (finishNotificationTask != null) {
                                return@withLock finishNotificationTask
                            }
                        }

                        notificationTask
                    } ?: continue

                    notificationTask()
                }
                catch (ignore: Exception) {}

                delay(1000)
            }
        }
    }

    suspend fun add(
        notificationId: Int,
        notificationType: NotificationEventType,
        notificationTask: () -> Unit
    ) {

        lock.withLock {
            val key = Pair(notificationId, notificationType)

            if (notifications.put(key, notificationTask) == null) {
                queue.send(key)
            }
        }
    }

    suspend fun cancelAll() {
        lock.withLock {
            notifications.clear()
        }
    }

    suspend fun cancel(id: Int) {
        lock.withLock {
            for (t in NotificationEventType.values()) {
                notifications.remove(Pair(id, t))
            }
        }
    }
}