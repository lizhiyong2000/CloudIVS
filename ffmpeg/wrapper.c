#include <libavcodec/avcodec.h>
#include <libavcodec/version.h>
#include <libavformat/avformat.h>
#include <libavformat/version.h>
#include <libavutil/avutil.h>
#include <libavutil/dict.h>
#include <libavutil/version.h>
#include <pthread.h>
#include <stdbool.h>
#include <stdlib.h>

const int ffmpeg_compiled_libavcodec_version = LIBAVCODEC_VERSION_INT;
const int ffmpeg_compiled_libavformat_version = LIBAVFORMAT_VERSION_INT;
const int ffmpeg_compiled_libavutil_version = LIBAVUTIL_VERSION_INT;

const int ffmpeg_av_dict_ignore_suffix = AV_DICT_IGNORE_SUFFIX;

const int64_t ffmpeg_av_nopts_value = AV_NOPTS_VALUE;

const int ffmpeg_avmedia_type_video = AVMEDIA_TYPE_VIDEO;

const int ffmpeg_av_codec_id_h264 = AV_CODEC_ID_H264;

const int ffmpeg_averror_eof = AVERROR_EOF;

// Prior to libavcodec 58.9.100, multithreaded callers were expected to supply
// a lock callback. That release deprecated this API. It also introduced a
// FF_API_LOCKMGR #define to track its removal:
//
// * older builds (in which the lock callback is needed) don't define it.
// * middle builds (in which the callback is deprecated) define it as 1.
//   value of 1.
// * future builds (in which the callback removed) will define
//   it as 0.
//
// so (counterintuitively) use the lock manager when FF_API_LOCKMGR is
// undefined.

#ifndef FF_API_LOCKMGR
static int lock_callback(void **mutex, enum AVLockOp op) {
    switch (op) {
        case AV_LOCK_CREATE:
            *mutex = malloc(sizeof(pthread_mutex_t));
            if (*mutex == NULL)
                return -1;
            if (pthread_mutex_init(*mutex, NULL) != 0)
                return -1;
            break;
        case AV_LOCK_DESTROY:
            if (pthread_mutex_destroy(*mutex) != 0)
                return -1;
            free(*mutex);
            *mutex = NULL;
            break;
        case AV_LOCK_OBTAIN:
            if (pthread_mutex_lock(*mutex) != 0)
                return -1;
            break;
        case AV_LOCK_RELEASE:
            if (pthread_mutex_unlock(*mutex) != 0)
                return -1;
            break;
        default:
            return -1;
    }
    return 0;
}
#endif

void ffmpeg_init(void) {
#ifndef FF_API_LOCKMGR
    if (av_lockmgr_register(&lock_callback) < 0) {
        abort();
    }
#endif
}

struct ffmpeg_streams {
    AVStream** streams;
    size_t len;
};

struct ffmpeg_data {
    uint8_t *data;
    size_t len;
};

struct ffmpeg_streams ffmpeg_fctx_streams(AVFormatContext *ctx) {
    struct ffmpeg_streams s = {ctx->streams, ctx->nb_streams};
    return s;
}

AVPacket *ffmpeg_packet_alloc(void) { return malloc(sizeof(AVPacket)); }
void ffmpeg_packet_free(AVPacket *pkt) { free(pkt); }
bool ffmpeg_packet_is_key(AVPacket *pkt) { return (pkt->flags & AV_PKT_FLAG_KEY) != 0; }
int64_t ffmpeg_packet_pts(AVPacket *pkt) { return pkt->pts; }
void ffmpeg_packet_set_dts(AVPacket *pkt, int64_t dts) { pkt->dts = dts; }
void ffmpeg_packet_set_pts(AVPacket *pkt, int64_t pts) { pkt->pts = pts; }
void ffmpeg_packet_set_duration(AVPacket *pkt, int dur) { pkt->duration = dur; }
int64_t ffmpeg_packet_dts(AVPacket *pkt) { return pkt->dts; }
int ffmpeg_packet_duration(AVPacket *pkt) { return pkt->duration; }
int ffmpeg_packet_stream_index(AVPacket *pkt) { return pkt->stream_index; }
struct ffmpeg_data ffmpeg_packet_data(AVPacket *pkt) {
    struct ffmpeg_data d = {pkt->data, pkt->size};
    return d;
}

AVCodecParameters *ffmpeg_stream_codecpar(AVStream *stream) { return stream->codecpar; }
AVRational ffmpeg_stream_time_base(AVStream *stream) { return stream->time_base; }

int ffmpeg_codecpar_codec_id(AVCodecParameters *codecpar) { return codecpar->codec_id; }
int ffmpeg_codecpar_codec_type(AVCodecParameters *codecpar) {
    return codecpar->codec_type;
}
struct ffmpeg_data ffmpeg_codecpar_extradata(AVCodecParameters *codecpar) {
    struct ffmpeg_data d = {codecpar->extradata, codecpar->extradata_size};
    return d;
}
int ffmpeg_codecpar_height(AVCodecParameters *codecpar) { return codecpar->height; }
int ffmpeg_codecpar_width(AVCodecParameters *codecpar) { return codecpar->width; }
