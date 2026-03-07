package com.plugin.android_fs

import android.annotation.SuppressLint
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import app.tauri.annotation.InvokeArg
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
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

        @Synchronized
        private fun getOrInitNotificationQueueManager(scope: CoroutineScope): NotificationQueueManager {
            return notificationQueueManager ?: NotificationQueueManager(scope).also { notificationQueueManager = it }
        }

        @Synchronized
        private fun setProgressNotificationChannelIfNeed(ctx: Context) {
            if (isNotifiedProgressChannel) return

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

        @SuppressLint("MissingPermission")
        fun startProgressNotification(
            iconType: ProgressNotificationIconType,
            title: String?,
            text: String?,
            subText: String?,
            progressMax: Int?,
            progress: Int?,
            ctx: Context,
            scope: CoroutineScope,
        ): Int {

            setProgressNotificationChannelIfNeed(ctx)

            val id = notificationIdCounter.incrementAndGet()
            notifications.add(id)

            getOrInitNotificationQueueManager(scope).add(NotificationEvent.Start {
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
            })

            return id
        }

        @SuppressLint("MissingPermission")
        fun updateProgressNotification(
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

            notificationQueueManager?.add(NotificationEvent.Update {
                if (!notifications.contains(id)) return@Update

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

                if (!notifications.contains(id)) return@Update
                NotificationManagerCompat.from(ctx).notify(id, builder.build())
            })
        }

        @SuppressLint("MissingPermission")
        fun finishProgressNotification(
            id: Int,
            iconType: ProgressNotificationIconType,
            title: String?,
            text: String?,
            subText: String?,
            error: Boolean,
            ctx: Context,
        ) {

            notifications.remove(id)

            notificationQueueManager?.add(NotificationEvent.Finish {
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
            })
        }
    }
}

private sealed class NotificationEvent {
    abstract val run: () -> Unit

    data class Start(override val run: () -> Unit): NotificationEvent()
    data class Update(override val run: () -> Unit): NotificationEvent()
    data class Finish(override val run: () -> Unit): NotificationEvent()
}

/**
 * Android ではパッケージ単位での通知の送信・更新にレート制限があるため、
 * 優先度と遅延付きの Queue で通知を処理する。
 * https://saket.me/android-7-nougat-rate-limiting-notifications/
 */
private class NotificationQueueManager(scope: CoroutineScope) {
    private val channel = Channel<NotificationEvent>(Channel.UNLIMITED)
    private val pendingCount = AtomicInteger(0)

    init {
        scope.launch {
            for (event in channel) {
                val pc = pendingCount.getAndDecrement()

                // イベントが Update で、かつ未処理のイベントが一定以上ある場合はスキップ
                if (event is NotificationEvent.Update && 3 < pc) {
                    continue
                }

                try {
                    event.run()
                }
                catch (ignore: Exception) {}

                delay(1000)
            }
        }
    }

    fun add(event: NotificationEvent) {
        pendingCount.incrementAndGet()
        if (!channel.trySend(event).isSuccess) {
            pendingCount.decrementAndGet()
        }
    }
}