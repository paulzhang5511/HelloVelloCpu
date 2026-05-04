package com.example.demo

import android.content.Context
import android.util.AttributeSet
import android.view.Choreographer
import android.view.MotionEvent
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView

class VelloSurfaceView @JvmOverloads constructor(
    context: Context, attrs: AttributeSet? = null
) : SurfaceView(context, attrs), SurfaceHolder.Callback, Choreographer.FrameCallback {

    companion object {
        init {
            // 加载用户的业务库 (它内部包含了 rustlib)
            System.loadLibrary("rustlib")
        }
    }

    // --- 严格对应 rustlib 动态注册的 JNI 方法 ---
    private external fun nativeInit()
    private external fun nativeSurfaceCreated(surface: Surface)
    private external fun nativeSurfaceChanged(width: Int, height: Int)
    private external fun nativeSurfaceDestroyed()
    private external fun nativeDoFrame(frameTimeNanos: Long)
    private external fun nativeTouchEvent(action: Int, x: Float, y: Float)

    init {
        holder.addCallback(this)
        nativeInit() // 触发底层的初始化和后台线程启动
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        nativeSurfaceCreated(holder.surface)
        // 启动 Choreographer 高帧率动画循环
        Choreographer.getInstance().postFrameCallback(this)
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        nativeSurfaceChanged(width, height)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        nativeSurfaceDestroyed()
        Choreographer.getInstance().removeFrameCallback(this)
    }

    override fun doFrame(frameTimeNanos: Long) {
        if (holder.surface.isValid) {
            nativeDoFrame(frameTimeNanos)
            Choreographer.getInstance().postFrameCallback(this)
        }
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        // 屏蔽多点触控，只传主手指的坐标
        nativeTouchEvent(event.actionMasked, event.x, event.y)
        return true
    }
}