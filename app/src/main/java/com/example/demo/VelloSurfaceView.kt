package com.example.demo

import android.content.Context
import android.graphics.PixelFormat
import android.util.AttributeSet
import android.util.Log
import android.view.Choreographer
import android.view.MotionEvent
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView

// SurfaceView 类，用于与 Rust 渲染引擎交互
class VelloSurfaceView @JvmOverloads constructor(
    context: Context, attrs: AttributeSet? = null
) : SurfaceView(context, attrs), SurfaceHolder.Callback, Choreographer.FrameCallback {

    companion object {
        // 日志标签
        private const val TAG = "VelloSurfaceView"
        
        init {
            Log.i(TAG, "[Companion::init] Loading native library: rustlib")
            // 加载用户的业务库 (它内部包含了 rustlib)
            System.loadLibrary("rustlib")
            Log.i(TAG, "[Companion::init] Native library loaded successfully")
        }
    }

    // --- 严格对应 rustlib 动态注册的 JNI 方法 ---
    // 原生初始化方法
    private external fun nativeInit()
    // 原生 Surface 创建方法
    private external fun nativeSurfaceCreated(surface: Surface)
    // 原生 Surface 尺寸变化方法
    private external fun nativeSurfaceChanged(width: Int, height: Int)
    // 原生 Surface 销毁方法
    private external fun nativeSurfaceDestroyed()
    // 原生帧渲染方法
    private external fun nativeDoFrame(frameTimeNanos: Long)
    // 原生触摸事件处理方法
    private external fun nativeTouchEvent(action: Int, x: Float, y: Float)

    // 标志位，用于跟踪帧回调是否已注册
    private var frameCallbackRegistered = false

    init {
        Log.d(TAG, "[init] Initializing VelloSurfaceView")
        holder.setFormat(PixelFormat.RGBA_8888)
        Log.d(TAG, "[init] SurfaceHolder format set to RGBA_8888")
        holder.addCallback(this)
        Log.d(TAG, "[init] SurfaceHolder callback added")
        
        Log.d(TAG, "[init] Calling nativeInit()")
        nativeInit() // 触发底层的初始化和后台线程启动
        Log.d(TAG, "[init] nativeInit() completed")
        Log.i(TAG, "[init] VelloSurfaceView initialized successfully")
    }

    // Surface 创建回调
    override fun surfaceCreated(holder: SurfaceHolder) {
        Log.i(TAG, "[surfaceCreated] Surface created event received")
        Log.d(TAG, "[surfaceCreated] Surface: ${holder.surface}")
        Log.d(TAG, "[surfaceCreated] Surface is valid: ${holder.surface.isValid}")
        
        Log.d(TAG, "[surfaceCreated] Calling nativeSurfaceCreated()")
        nativeSurfaceCreated(holder.surface)
        Log.d(TAG, "[surfaceCreated] nativeSurfaceCreated() completed")
        
        // 启动 Choreographer 高帧率动画循环
        Log.d(TAG, "[surfaceCreated] Registering frame callback with Choreographer")
        Choreographer.getInstance().postFrameCallback(this)
        frameCallbackRegistered = true
        Log.i(TAG, "[surfaceCreated] Frame callback registered successfully")
    }

    // Surface 尺寸变化回调
    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        Log.i(TAG, "[surfaceChanged] Surface changed event received")
        Log.d(TAG, "[surfaceChanged] Format: $format, Width: $width, Height: $height")
        Log.d(TAG, "[surfaceChanged] Surface: ${holder.surface}")
        Log.d(TAG, "[surfaceChanged] Surface is valid: ${holder.surface.isValid}")
        
        Log.d(TAG, "[surfaceChanged] Calling nativeSurfaceChanged($width, $height)")
        nativeSurfaceChanged(width, height)
        Log.d(TAG, "[surfaceChanged] nativeSurfaceChanged() completed")
        Log.i(TAG, "[surfaceChanged] Surface change handled successfully")
    }

    // Surface 销毁回调
    override fun surfaceDestroyed(holder: SurfaceHolder) {
        Log.w(TAG, "[surfaceDestroyed] Surface destroyed event received")
        Log.d(TAG, "[surfaceDestroyed] Surface: ${holder.surface}")
        
        Log.d(TAG, "[surfaceDestroyed] Calling nativeSurfaceDestroyed()")
        nativeSurfaceDestroyed()
        Log.d(TAG, "[surfaceDestroyed] nativeSurfaceDestroyed() completed")
        
        // 移除帧回调
        if (frameCallbackRegistered) {
            Log.d(TAG, "[surfaceDestroyed] Removing frame callback from Choreographer")
            Choreographer.getInstance().removeFrameCallback(this)
            frameCallbackRegistered = false
            Log.d(TAG, "[surfaceDestroyed] Frame callback removed successfully")
        } else {
            Log.w(TAG, "[surfaceDestroyed] Frame callback was not registered, skipping removal")
        }
        
        Log.w(TAG, "[surfaceDestroyed] Surface destruction handled successfully")
    }

    // Choreographer 帧回调
    override fun doFrame(frameTimeNanos: Long) {
        Log.v(TAG, "[doFrame] Frame callback received")
        Log.v(TAG, "[doFrame] Frame time nanos: $frameTimeNanos")
        
        if (holder.surface.isValid) {
            Log.v(TAG, "[doFrame] Surface is valid, calling nativeDoFrame()")
            nativeDoFrame(frameTimeNanos)
            Log.v(TAG, "[doFrame] nativeDoFrame() completed")
            
            Log.v(TAG, "[doFrame] Posting next frame callback")
            Choreographer.getInstance().postFrameCallback(this)
            Log.v(TAG, "[doFrame] Next frame callback posted successfully")
        } else {
            Log.w(TAG, "[doFrame] Surface is not valid, skipping frame rendering and next callback")
        }
    }

    // 触摸事件处理
    override fun onTouchEvent(event: MotionEvent): Boolean {
        Log.v(TAG, "[onTouchEvent] Touch event received")
        Log.v(TAG, "[onTouchEvent] Action: ${event.actionMasked}, X: ${event.x}, Y: ${event.y}")
        Log.v(TAG, "[onTouchEvent] Event details: $event")
        
        // 屏蔽多点触控，只传主手指的坐标
        Log.v(TAG, "[onTouchEvent] Calling nativeTouchEvent(${event.actionMasked}, ${event.x}, ${event.y})")
        nativeTouchEvent(event.actionMasked, event.x, event.y)
        Log.v(TAG, "[onTouchEvent] nativeTouchEvent() completed")
        
        return true
    }
}
