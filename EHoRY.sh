#!/bin/sh
#################################################################################################################################
# Create By yu13140 [whmyc801@gmail.com],
SCRIPT_VERSION="v5.1"
PATCH_VERSION="Iceman"

# 颜色定义
RED='\033[1;31m'
GR='\033[1;32m'
YE='\033[1;33m'
RE='\033[0m'
WH='\033[1;37m'
BLUE='\033[1;34m'
WA="${BLUE}*${GR}"

# 变量区
YSHELL_PATH="/data/cache/recovery/yshell"
MODULE_DE="$YSHELL_PATH/installmodule.zip"

# 检测当前环境 (注意，这样的检测是不完全正确的)
detect_environment() {
ENVIRONMENT=""
KERNELSU_FLAG=0
APATCH_FLAG=0
MAGISK_FLAG=0

[[ -d "/data/adb/ksu" ]] || [[ -f "/data/adb/ksud" ]] && KERNELSU_FLAG=1
[[ -d "/data/adb/ap" ]] || [[ -f "/data/adb/apd" ]] && APATCH_FLAG=1
[[ -d "/data/adb/magisk" ]] || [[ -f "/data/adb/magisk.db" ]] && MAGISK_FLAG=1

ENV_COUNT=$(( KERNELSU_FLAG + APATCH_FLAG + MAGISK_FLAG ))

if [[ $ENV_COUNT -gt 1 ]]; then
    echos "$WH  "
    KERNELSUTAG=0
    APATCHTAG=0
    MAGISKTAG=0
    DMESGLOG="$YSHELL_PATH/dmesg.log"
    dmesg > "$DMESGLOG"
    grep -q -F "KP I commit_common_su" "$DMESGLOG" && APATCHTAG=1
    command -v magisk && MAGISKTAG=1
    grep -q -F "KernelSU" "$DMESGLOG" && KERNELSUTAG=1
    COUNT=$(( KERNELSUTAG + APATCHTAG + MAGISKTAG ))
    if [[ $COUNT -eq 3 ]]; then
        echos "- 究极逆天环境：检测到了Magisk和KSU和APatch在你的设备上"
        exit 1
    eles
        if [[ $KERNELSU_FLAG -eq 1 ]] && [[ $APATCH_FLAG -eq 1 ]]; then
            [[ $COUNT -eq 2 ]] && echos "- 逆天环境：检测到了KSU和APatch共存" && exit 1 || {
                echos "经过综合判断，未存在Root共存问题，但是存在一些残留文件，正在为您清理"
                [[ $KERNELSUTAG -eq 1 ]] && rm -rf /data/adb/ap /data/adb/apd
                [[ $APATCHTAG -eq 1 ]] && rm -rf /data/adb/ksu /data/adb/ksud
            }
        elif [[ $KERNELSU_FLAG -eq 1 ]] && [[ $MAGISK_FLAG -eq 1 ]]; then
            [[ $COUNT -eq 2 ]] && echos "- 逆天环境：检测到了KSU和Magisk共存" && exit 1 || {
                echos "经过综合判断，未存在Root共存问题，但是存在一些残留文件，正在为您清理"
                [[ $KERNELSUTAG -eq 1 ]] && rm -rf /data/adb/magisk /data/adb/magisk.db
                [[ $MAGISKTAG -eq 1 ]] && rm -rf /data/adb/ksu /data/adb/ksud
            }
        elif [[ $MAGISK_FLAG -eq 1 ]] && [[ $APATCH_FLAG -eq 1 ]]; then
            [[ $COUNT -eq 2 ]] && echos "- 逆天环境：检测到了Magisk和APatch共存" && exit 1 || {
                echos "经过综合判断，未存在Root共存问题，但是存在一些残留文件，正在为您清理"
                [[ $APATCHTAG -eq 1 ]] && rm -rf /data/adb/magisk /data/adb/magisk.db
                [[ $MAGISKTAG -eq 1 ]] && rm -rf /data/adb/ap /data/adb/apd
            }
        fi
    fi
fi

if [[ $KERNELSU_FLAG -eq 1 ]]; then
    export ENVIRONMENT="KernelSU"
    BUSYBOX_PATH="/data/adb/ksu/bin/busybox"
    KSU_VERSION="$(/data/adb/ksud debug version | sed 's/Kernel Version: //g')"
    KSU_V="$(/data/adb/ksud -V | sed 's/ksud //g' | cut -d "-" -f1)"
    if [[ $KSU_V == "zako"* ]]; then
        export ENVIRONMENT="SukiSU"
    fi
    BUSY="/data/adb/ksu/bin/busybox"
elif [[ $APATCH_FLAG -eq 1 ]]; then
    export ENVIRONMENT="APatch"   
    BUSYBOX_PATH="/data/adb/ap/bin/busybox"
    APATCH_VERSION="$(sed -n '1p' /data/adb/ap/version 2>/dev/null)"

elif [[ $MAGISK_FLAG -eq 1 ]]; then
    export ENVIRONMENT="Magisk"
    BUSYBOX_PATH="/data/adb/magisk/busybox"
    MAGISK_VERSION="$(magisk -V 2>/dev/null)"
    MAGISK_FORK="$(magisk -v 2>/dev/null)"  
fi

[[ -z "$ENVIRONMENT" ]] && echos "$WH- 警告：未检测到Root环境$RE" >&2
}

# 简化调用Cli时的命令
downloader() { rshy --download "$1" "$MODULE_DE" "$2" ; }
download_module() { rshy --download "$1" "$MODULE_DE" "$2" --no-cdn ; }
pmx() { rshy --tools cmd package $1 $2 ; }

detect_magisk() {
if [[ $ENVIRONMENT = "Magisk" ]]; then
    if [[ $(rshy --zygiskcheck) = 1 ]]; then
        echos "$YE检测到您开启了magisk自带的zygisk"
        echos "建议您关闭它以获得最佳的隐藏状态$RE"
        exit 1
    fi
fi
}

# 安装模块
installer() {
chmod 755 "$MODULE_DE"
case $ENVIRONMENT in
    Magisk) magisk --install-module "$MODULE_DE" ;;
    APatch) /data/adb/apd module install "$MODULE_DE" ;;
    KernelSU | SukiSU) /data/adb/ksud module install "$MODULE_DE" ;;
esac
}

# 简化输出
echos() { echo -e "$@"; }

# 选择函数
qchoice() {
    local prompt="$1" cmd="$2"
    
    if [[ -n $3 ]]; then
        local option="$3"
    else
        local option="$1"
    fi
    
    echos " "    
    if [[ -n $4 ]]; then
        echos "$prompt"
    else
        echos "$YE是否安装$prompt模块？"
    fi
    
    echos "  1) 安装"
    echos "  2) 不安装"
    echos "请输入对应的选项：$RE\c"    
    
    read -r user_input
    
    case "$user_input" in
        1) eval "$cmd" ;;
        2) echos "$YE你选择不安装$option模块$RE" ;;
        *) echos "$YE输入错误，默认不安装$RE" ;;
    esac
}

# 处理函数带来的的结果
ends() {
if [[ $? -ne 0 ]]; then
    echos "$WH可能有问题出现咯，请截图下来私信酷安@yu13140$RE"
    exit 0
else
    while true; do
        echos "$GR命令执行完成$RE"
        echos " "
        echos "----------------------------------------"
        echos "$YE是否继续操作？"
        echos "1. 重启手机"
        echos "2. 返回主菜单"
        echos "3.进入过检测软件界面"
        echos "4.进入更新模块界面"
        echos "5. 退出脚本$RE"
        echos "----------------------------------------"
        echos "$YE请输入选项：$RE\c"

        read dot
        case $dot in
            1)
            echos "$WH三秒后即将重启手机$RE"; sleep 3; reboot ;;
            2)
            clear; selmenu; break ;;
            3)
            clear; menu; break ;;
            4)
            clear; gmodules; break ;;
            5) 
            echos "$YE正在退出脚本$RE"; exit 0 ;;
            *) 
            echos "$YE输入错误❌请输入1到5的其中一个数字！"; echos "请等待2秒$RE" ;sleep 1.4; clear ;;
        esac
    done
fi
}

# 音量键开关开发者模式
Development() {
    Volume_key_monitoring(){
    local choose
    local branch
        while :;do
            choose="$(getevent -qlc 1|awk '{ print $3 }')"
            case "$choose" in
                KEY_VOLUMEUP)branch="0";;
                KEY_VOLUMEDOWN)branch="1";;
                *)continue
            esac
            echos "$branch"
            break
        done
}
echos  " "
echos  " "
echos "$WH- 本脚本制作于2024年12月21日"
echos "- 脚本作者:酷安用户 @yu13140"
echos "- 脚本功能:让你更方便地开关开发者模式$RE"
echos  "$GR按音量键＋:开启开发者模式"
echos  "按音量键－: 关闭开发者模式"
echos  "如果失效，请多按几次$RE"
echos  "$YE—————————————————————————————————————$RE"
if [[ $(Volume_key_monitoring) == 0 ]];then
    echos  "$GR正在为您开启开发者模式$RE"
    settings put global adb_enabled 1
    settings put global development_settings_enabled 1
else
    echos  "$GR正在为您关闭开发者模式$RE"
    settings put global adb_enabled 0
    settings put global development_settings_enabled 0
fi
}

# 关闭开发者模式
momodevelop() {
        clear
		echos "$GR正在关闭调试模式$RE"
		rshy --tools cmd settings put global adb_enabled 0
		rshy --tools cmd settings put global development_settings_enabled 0
		if [[ "$DEVELOP" = 1 ]]; then
			echos "$GR调试模式未关闭，红魔设备请不要使用此选项$RE"
			echos "$GR其他设备请联系酷安@yu13140$RE"
		else
		    sleep 1.4
		fi
}

# 检查安装shamiko的环境
download_shamiko() {
if [[ $ENVIRONMENT = "APatch" ]]; then
    echos "{$WH}APatch不需要Shamiko，跳过此步骤"
    echos "2秒后将回到更新模块界面$RE"
    sleep 1.4
    gmodules
fi

if [[ $ENVIRONMENT = "Magisk" ]] && [[ $Kistune == 1 ]]; then
    echos "$WH狐狸面具不需要Shamiko，跳过此步骤" 
    echos "2秒后将回到更新模块界面$RE"
    sleep 1.4
    gmodules
fi

select_modules "1"
qchoice "快速切换Shamiko模式" "select_modules \"19\""
}

downopenselinux() {
murl="https://lz.qaiu.top/d/lz/iXBKa368frid"
mhash="1911ddfdd0262f85ff24dbd03aab0d64e83a68a580ce7993d6b78017a761b183"
download_module "$murl" "$mhash"
installer
}

# 脚本操作选择
select_option() {
case $1 in
    a)
    clear; selmenu ;;    
    1)
    clear; echos "$GR正在切换Shamiko模式$RE"; rshy shamiko_pattern; ends ;;
    2)
    clear; echos "$GR正在清理LSPosed日志$RE"; rshy lsplog; ends ;;
    3)
    clear; echos "$GR正在清理Magisk日志$RE"; rshy magisklog; ends ;;
    4)
    clear; echos "$GR正在启动$RE"; Development; ends ;;
    5)
    clear; Momo ;;
    6)
    clear; native_test ;;
    7)
    clear; echos "$GR正在解决 Root|BootLoader|Magisk|Lsposed$RE"; rshy aptroot; ends ;;
    8)
    clear; echos "$GR正在匹配CTS配置文件$RE"; rshy ctsfix; ends ;;
    9)
    clear; echos "$GR正在删除有问题的文件夹$RE"; rshy rurudelete; ends ;;
    10)
    clear; echos "$GR正在解决SElinux Flag$RE"; downlockselinux; ends ;;
    11)
    clear; echos "$GR正在解决init.rc已被Magisk修改$RE"; rshy initrc; ends ;;
    12)
    clear; echos "$GR正在解决Miscellaneous Check (a)$RE"; rshy holmes somethingwrong; ends ;;
    13)
    clear; echos "$GR正在解决Something Wrong$RE"; rshy holmes somethingwrong; ends ;;
    14)
    clear; echos "$GR正在解决哈希值问题$RE"; rshy nativetest boothash; ends ;;
    15)
    clear; echos "$GR正在解决Miscellaneous Check (2)$RE"; rshy initrc; ends ;;
    16)
    clear; echos "$GR正在解决Found Injection (9ff)$RE"; rshy holmes 9ff; ends ;;
    17)
    clear; echos "$GR正在解决Detected LSPosed (5)$RE"; rshy nativedetector lsp5; ends ;;
    18)
    clear; echos "$GR正在检测到Boot状态异常问题$RE"; rshy nativedetector vbmeta; installer; ends ;;
    19)
    clear; echos "$GR正在解决Magic Mount泄露$RE"; rshy nativedetector magicmount; ends ;;
    20)
    clear; echos "$GR正在解决SafetyDetectClient Check Root/Unlock$RE"; rshy hunter manager; ends ;;
    21)
    clear; echos "$GR正在解决Find Risk File$RE"; rshy hunter shizuku; ends ;;
    22)
    clear; echos "$GR正在解决Bootloader锁问题$RE"; rshy updatetarget; ends ;;
    23)
    clear; echos "$GR正在解决Selinux权限泄露问题$RE"; select_modules "16"; ends ;;    
    24)
    clear; echos "$GR正在解决权限泄露问题$RE"; rshy awjclean; ends ;;
    25)
    clear; echos "$GR正在解决哈希值问题$RE"; rshy nativetest boothash; installer; ends ;;
    f)
    echos "$GR正在退出脚本$RE"; exit 0 ;;
    *) 
    echos "$YE输入错误，请重新选择$RE"; echos "$GR请等待2秒$RE"; sleep 1.4; menu ;;
esac
}

native_test() {
niutous=(
        "$YE- a.返回上一级$RE$GR"
        "1.Native Test：Futile Hide (10)(重启后可能失效)"
        "2.Native Test：Conventional Tests (8)+Partition Modified"    
        "3.Native Test：Futile Hide (01)"
        "4.Native Test：Conventional Tests (1)"        
        "5.Native Test：Evil Service (2)"
        "6.Native Test：Found Injection (04)"
        "7.${WA}Native Test：Futile Hide (8)$RE"
        )
        for nt in "${!niutous[@]}"; do
        echos "${niutous[$nt]}"
        sleep 0.007
        done
        echos "$YE请输入选项：$RE\c"
        read dent        	    
        case $dent in
        a)
        clear; menu ;;
        1) 
        clear; echos "$GR正在解决Futile Hide (10)$RE"; rshy nativetest futile10; ends ;;
        2) 
        clear; echos "$GR正在解决哈希值问题$RE"; rshy nativetest boothash; installer; ends ;;
        3) 
        clear; echos "$GR正在解决Futile Hide (01)$RE"; /data/adb/modules/zygisksu/bin/zygiskd enforce-denylist disabled; ends ;;
        4) 
        clear; echos "$GR正在解决Conventional Tests (1)$RE"; select_modules "11"; ends ;;
        5) 
        clear; echos "$GR正在解决Evil Service (2)$RE"; select_modules "8"; ends ;;
        6) 
        clear; echos "$GR正在解决Found Injection (04)$RE"; select_modules "2"; ends ;;
        7)
        clear; echos "$GR正在解决Futile Hide (8)$RE"; rshy --delete dir /data/adb/modules/playintegrityfix; select_modules "11"; ends ;;
        *) 
        echos "$GR输入错误，请重新输入$RE"; native_test ;;
        esac
}

Momo() {
Momos=(
        "$YE- a.返回上一级$RE$GR"
        "1.Momo：已开启调试模式"
        "2.Momo：init.rc被修改"
        "3.Momo：处于调试环境"
        "4.Momo：ART参数异常/SElinux处于宽容模式"
        "5.Momo：非SDK接口的限制失效"
        "6.Momo：数据未加密，挂载参数被修改"
        "7.Momo：设备正在使用非原厂系统"
        "8.Momo：tee损坏$RE"       
        )
        for m in "${!Momos[@]}"; do
        echos "${Momos[$m]}"
        sleep 0.007
        done
        echos "$YE请输入选项：$RE\c"
        read demo
        case $demo in
        a)
        clear; menu ;;
        1) 
        clear; echos "$GR正在关闭调试模式$RE"; momodevelop; ends ;;
        2) 
        clear; echos "$GR正在解决 init.rc被修改 问题$RE"; rshy initrc; installer; ends ;;
        3) 
        clear; echos "$GR解决 处于调试环境 问题$RE"; rshy momo development; installer; ends ;;
        4) 
        clear; echos "$GR正在为SElinux切换至默认模式$RE"; downopenselinux; ends ;;
        5) 
        clear; echos "$GR正在解决 非SDK接口的限制失效 问题$RE"; rshy momo sdk; ends ;;
        6) 
        clear; echos "$GR正在解决 数据未加密，挂载参数被修改 问题$RE"; rshy momo systemmount; installer; ends ;;
        7)
        clear; echos "$GR正在解决 设备正在使用非原厂系统 问题$RE"; rshy momo addon; installer; ends ;;
        8)
        clear; echos "$GR正在解决tee损坏$RE"; rshy momo tee; ends ;;      
        *) 
        echos "$GR输入错误，请重新输入$RE"; Momo ;;
        esac
}

menu() {
    clear   
     
    OPTIONS=(
        "$GR您正在使用 过检测软件 功能"
        "带${WA}的是实验性功能"
        "以下是隐藏功能列表：$RE$YE"
        "a.返回主菜单$RE$GR"
        "1.切换Shamiko模式"
        "2.清理LSPosed日志"
        "3.清理Magisk日志"
        "4.快捷开关 开发者模式$RE$YE"
        "5.解决Momo问题"
        "6.解决牛头 (Native Test) 问题$RE$GR"              
        "7.APT检测：Root|BootLoader|Magisk|Lsposed"
        "8.YASNAC/SPIC：匹配CTS配置文件❌"
        "9.Ruru：TWRP/XPrivacyLua/Xposed Edge"
        "10.RootbeerFresh：SElinux Flag is Enabled"
        "11.Magisk检测应用：init.rc已被Magisk修改"
        "${WA}12.Holmes：Miscellaneous Check (a)"
        "/Evil Modification (7)"
        "13.Holmes：Something Wrong"
        "14.Holmes：Property Modified (1)"
        "15.Holmes：Miscellaneous Check (2)"
        "${WA}16.Holmes：Found Injection (9ff)"
        "17.Native Detector：Detected LSPosed (5)"
        "18.Native Detector：检测到Boot状态异常"
        "19.Native Detector：Magic Mount"
        "20.Hunter：SafetyDetectClient Check [Root/Unlock]"
        "21.Hunter：Find Risk File(shizuku)"
        "22.Hunter：当前手机 已经被解锁/Boot分区签名失败"
        "23.Hunter：Find prop Modify Mark"
        "24.Luna：发现风险应用"
        "25.Luna：证书 系统安全性被破坏$RE"
    )

    for i in "${!OPTIONS[@]}"; do
        echos "${OPTIONS[$i]}"
        sleep 0.001
    done
    echos "$YE(输入 f 退出脚本)请输入选项：$RE\c"
    read yc
    select_option $yc
}

gmodules() {
MODULELS=(
        " "
        "$GR您正在使用 更新模块 功能"
        "请选择你要更新的模块"
        " "
        "${YE}a.返回主界面$RE"
        "${GR}1.Shamiko v1.2.5.1 (417)"
        "2.Zygisk Next 1.3.0-RC3 (620)"
        "3.ReZygisk v1.0.0 (422)"
        "4.Treat Wheel v0.0.4"
        "5.NeoZygisk v2.2 (266)"
        "6.Tricky Store v1.4.0 (235)"
        "7.Tricky Store OSS v2.1.0 (69)"
        "8.LSPosed v1.9.2-it (7419)"
        "9.LSPosed-JingMatrix v1.10.1 (7190)"
        "10.LSPosed-Irena 1.9.2 (7291)"
        "11.PlayIntegrityFork-v14"
        "12.Play Integrity Fix [INJECT] v4.3"
        "13.TS Enhancer Extreme v0.8.3-RC1"
        "14.TrickyAddonModule-v4.2"
        "15.nohello v0.0.7 (58)"
        "16.Zygisk Maphide 2.0"
        "17.ZLoader Next 0.1.3"
        "18.切换Shamiko模式+添加包名(模块)"
        "19.自动神仙救砖模块"
        "20.自动添加包名模块(试用)"
        "21.cherish_peekaboo嵌入"
        "22.Hide_Bootloader v1"
    )

    clear
    for mods in "${!MODULELS[@]}"; do
        echos "${MODULELS[$mods]}"
        sleep 0.01
    done
    echos "$YE(输入 f 退出脚本)请输入选项：$RE\c"
    read yctw
    select_modules $yctw
}

# 存放模块的哈希值
module_hash() {
case $1 in
    1) mhash="308d31b2f52a80e49eb58f46bc4c764a6588a79e4b8d101b44860832023f88b4" ;;
    2) mhash="5c51172a3ee221985d0f34637be18a3bb3983d0ee2c1143cd5b032d252aca0b7" ;;
    3) mhash="c6868663cccf43a4cdc03c0d8fa881e1aa467c858da0af9f0a597e57c7bc64f3" ;;
    4) mhash="d38a4d0176327ec990c8993bbed84c1bd28820d98443541b40df04fb0d3c3f70" ;;
    5) mhash="553f54627f5ae277c35d506ac684148bc85d31b7ba8daf33d29770e22a5e5271" ;;
    6) mhash="1c923b8f003142bddcfc0c64de98ccb0df0631f54a0e684c8f9d10f4ff466bc3" ;;
    7) mhash="b7b564fae5d0e70dc9bf61c1bab6caa84370304166b8baf96d73537d84a648c8" ;;
    8) mhash="b338af381600e718962c2a7d83e10a1c6a8f66ec7f6c878638e115801e3f197b" ;;
    9) mhash="b726cb05f6846b2c3506497a0d6d000e50f56f6eb148e01124d58a90d35310b6" ;;
    10) mhash="ade3ec2550ecdd67956e128656ca15ae38b5dcb20aa05a7a34fd0f04491a46e8" ;;
    11) mhash="b75bacf7a9d169d797af13eba3ac0252d3c406b175c7b187d882296e0f7cde89" ;;
    12) mhash="72e51d12d7f3df0516b4e4396b1a24ab1bdc90799d447e22cc4572939afcd718" ;;
    13) mhash="bc966a7eb41469bde112c8b6de8134af9497a778ab1c30a998ceb50344392e89" ;;
    14) mhash="f75977cbeb46a2d75b5689fc9dabbd8ad0c55e36d4cb8d657da8e9d70517fcb7" ;;
    15) mhash="0df200255651feed840c2cea353bdc265df008c31409fcb85bacd54c607f651a" ;;
    16) mhash="c58d4c7cdaf8c5137d6b1b37ad9c4ed5796d891ab79832745130df97278677d0" ;;
    17) mhash="0b5f4145e1dcf27834be135cb3b5957446d14ab881ca7f31e84a6acedd8ab053" ;;
    18) mhash="75ec17e7a133a3627c576108c380cacc45bf730734d2b10243ba4832fdc411cc" ;;
    19) mhash="c2fe31adbd4d4ef08fb9d887bcee484005379d1e7641f2075f98365170bd88b8" ;;
    20) mhash="c46f0139ba466f98a18fd79c8f2219974f85b5cda59cf626cf2bccc710a72d83" ;;
    22) mhash="beae621fc686862894dc51c15b1686dad7603d7689f365ecc6470b143c9c391e" ;;
esac
}

# 存放下载模块的链接
module_url() {
case $1 in
    1) share_key="ihNR035paq3i" ;;
    2) share_key="iexT03971ltg" ;;
    3) share_key="ivqzS3977mqf" ;;
    4) share_key="inl0b351ibze" ;;
    5) share_key="imRo63971m0d" ;;
    6) share_key="ixHA439788gh" ;;
    7) share_key="ibCGy353muze" ;;
    8) share_key="iNQkz3971eej" ;;
    9) share_key="/ivqzS3977mqf" ;;
    10) share_key="iknEl368gyib" ;;
    11) share_key="imnUV351ibfe" ;;
    12) share_key="iXMep35b56lc" ;;
    13) share_key="iZuiA3971mgj" ;;
    14) share_key="iOyfq3971lyb" ;;
    15) share_key="iZF4P351ibdc" ;;
    16) share_key="iyOP7351icid" ;;
    17) share_key="iQIyW351icfa" ;;
    18) share_key="iCU9o351ibih" ;;
    19) share_key="iRR0r351icvg" ;;
    20) share_key="id0lL351icuf" ;;
    22) share_key="i8HY8351icza" ;;
esac
murl="https://lz.qaiu.top/d/lz/$share_key"
}

# 下载提供的模块
select_modules() {
module_info() {
module_hash "$1"
module_url "$1"
download_module "$murl" "$mhash"
installer
ends
}
case $1 in
    a)
    clear; selmenu ;;    
    1)
    clear; echos "$GR正在下载Shamiko$RE"; download_shamiko; ends ;;
    [235])
    clear; detect_magisk; echos "$GR正在下载Zygisk模块$RE"; module_info "$1" ;;
    [46-9]|1[0-9]|20|22)
    clear; echos "$GR正在下载模块$RE"; module_info "$1" ;;
    21)
    clear; ddpeekaboo; ends ;;
    f)
    echos "$GR正在退出脚本$RE"; exit 0 ;;
    *)
    echos "$YE输入错误，请重新选择$RE"; echos "$GR请等待2秒$RE"; sleep 1.4; gmodules ;;
esac
}

selmenu() {
clear
# 选择root管理器
    if [[ $ENVIRONMENT = "Magisk" ]]; then
        if echos "$MAGISK_FORK" | grep -qi "kitsune"; then
	    	MANAGER="$YE检测到你已root
你的root管理器为Kitsune Mask("$MAGISK_VERSION")$RE"
            Kistune=1
        elif echos "$MAGISK_FORK" | grep -qi "alpha"; then
            MANAGER="$YE检测到你已root
你的root管理器为Magisk Alpha("$MAGISK_VERSION")$RE"
        else
            MANAGER="$YE检测到你已root
你的root管理器为Magisk("$MAGISK_VERSION")$RE"
        fi
	elif [[ $ENVIRONMENT = "KernelSU" ]]; then
		MANAGER="$YE检测到你已root
你的root管理器为KernelSU("$KSU_VERSION")$RE"
    elif [[ $ENVIRONMENT = "SukiSU" ]]; then
		MANAGER="$YE检测到你已root
你的root管理器为SukiSU("$KSU_VERSION")$RE"
    elif [[ $ENVIRONMENT = "APatch" ]]; then
    APATCH_NEXT_VERSIONS="11008 11010 11021"
        if echos " $APATCH_NEXT_VERSIONS " | grep -q " $APATCH_VERSION "; then
            MANAGER="$YE检测到你已root
你的root管理器为APatch Next($APATCH_VERSION)$RE"
            NEXT_AP=1
        else
            MANAGER="$YE检测到你已root
你的root管理器为APatch($APATCH_VERSION)$RE"
            NEXT_AP=0
        fi
	else
		MANAGER="$YE检测到你已root
未知root管理器，请谨慎使用脚本$RE"
	fi

yiyan=$(rshy --yiyan)

MENU=(
        " "
       "$GR————————————————————————————————————————————————"
        ".########.##.....##..#######..########..##....##."
        ".##.......##.....##.##.....##.##.....##..##..##.."
        ".##.......##.....##.##.....##.##.....##...####..."
        ".######...#########.##.....##.########.....##...."
        ".##.......##.....##.##.....##.##...##......##...."
        ".##.......##.....##.##.....##.##....##.....##...."
        ".########.##.....##..#######..##.....##....##...."
       "————————————————————————————————————————————————"
        " "
        "$WH每日一言："
        "${yiyan}${RE}"
        "${YE}AUTHOR：酷安@yu13140$RE"
        "$YE当前脚本的版本号为：$SCRIPT_VERSION$RE"
        "$MANAGER"
        " "
        "$GR请选择你需要使用的功能"
        "1.过检测软件功能"
        " "
        "2.更新模块功能"
        " "
        "3.配置隐藏应用列表(可过Luna)"
        " "
        "4.一键安装隐藏所需模块"
        " "
        "5.一键安装检测软件10件套"
        " "
        "6.安装当前设备上的指定模块"
        " "
        "7.一键配置sus路径"
        " "
        "8.(实验)切换Root方案"
        " $RE"
    )
    
    for act in "${!MENU[@]}"; do
        echos "${MENU[$act]}"
        sleep 0.009
    done
    echos "$RE$YE(输入 f 退出脚本)请输入选项以执行下一步：$RE\c"
    read ssmenu
    menuss $ssmenu
}

# 安装检测软件
installapks() {
echos " "
echos "$YE正在下载必要文件中$RE"
downloader "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/apk.zip" "a826a0723b2313a3270a7c064b32b35f4b19b4344a82e009e96099699cbfc779"
    
    sleep 0.1
    echos "                                        "
    pmx uninstall icu.nullptr.nativetest >/dev/null 2>&1
    pmx uninstall com.android.nativetest >/dev/null 2>&1
    pmx uninstall com.reveny.nativecheck >/dev/null 2>&1
    pmx uninstall com.zhenxi.hunter >/dev/null 2>&1
    pmx uninstall me.garfieldhan.holmes >/dev/null 2>&1
    echos "${YE}正在为您安装检测软件$RE"
    unzip -o ""$MODULE_DE"" -d "/data/cache/recovery/yshell/apks/"
    for apk_file in /data/cache/recovery/yshell/apks/*; do
        if [[ -f "$apk_file" ]] && echo "$apk_file" | grep -iqE '\.apk$'; then
            apk_name="$(basename "$apk_file" .apk)"
            install_output=$(pmx install "$apk_file")
            if echo "$install_output" | grep -iq "Success"; then
                echos "$WH${apk_name}安装完成$RE"
            else
                echos "$WH${apk_name}安装失败"
            fi
        fi
    done
}

# 自定义安装本地模块
extra_module() {
extraout() {
        echos "1.回到主菜单       2.继续输入         3.退出此脚本$RE"
	    echos " "
	    echos "$GR请输入选项：$RE\c"

	    read whextra
	    case $whextra in
            1) 
            clear; selmenu ;;
            2) 
            clear; extramodule ;;
            3) 
            echos "$GR正在退出脚本……$RE"; sleep 1; exit 0 ;;
            *)
            echos "$GR输入错误，默认返回主菜单$RE"; clear; selmenu ;; 
        esac
}

IMODULELIST=(
           "$GR你正在使用 安装设备上的指定模块 功能$RE"
           " "
           "$GR你可以输入一个文件夹，自动识别该文件夹内的压缩包"
           "需要安装的模块的上一级路径，请确认没有同名文件夹或文件，不然可能会出现问题" 
           "你也可以输入需要安装的模块的绝对路径"
           " $RE"
	       )
	for exm in "${!IMODULELIST[@]}"; do
        echos "${IMODULELIST[$exm]}"
        sleep 0.07
    done
    echos "$YE请输入你需要安装的模块的路径"
    echos "路径：$RE\c"   
	read wmo
	if [[ -d $wmo ]]; then
	    MODULE_DE=""
	    module_found=0
	    for MODULE_DE in "$wmo"/*; do	        
	        if [[ -f "$MODULE_DE" ]] && echo "$MODULE_DE" | grep -iqE '\.zip$'; then
	            installer
	            module_found=1
	            if [[ $? -ne 0 ]]; then
	                echos "$YE模块：$MODULE_DE安装失败$RE"   
	            fi
	        fi	        
	    done
	    if [[ $module_found -eq 0 ]]; then	    
	        echos "$YE你输入的文件夹里好像没有压缩包呢，需要退出此功能吗$RE"
	        extra_out    
	    fi    
	    MODULE_DE="$YSHELL_PATH/installmodule.zip"
	    ends
	elif [[ -f $wmo ]]; then
	    MODULE_DE="$wmo"
	    installer
	    MODULE_DE="$YSHELL_PATH/installmodule.zip"
	    ends
        if [[ $? -ne 0 ]]; then
	        echos "$YE模块：$MODULE_DE安装失败$RE"   
	    fi
	else
	    echos "$YE你输入的好像不是一个压缩包或者一个文件夹呢，需要退出此功能吗$RE"
	    extra_out
    fi	      
}

chlist() {
NUMLIST=(
           "$GR你正在使用 配置隐藏应用列表 功能"
           "此功能理论上适配所有的隐藏应用列表(包括改包名版)"
           "不支持的应用，会把配置文件下载到/sdcard/Download/文件夹内"
           "1.配置隐藏应用列表"
           "2.恢复原配置"
           "3.退出脚本$RE"
	       "                                        "
	       )
	for coh in "${!NUMLIST[@]}"; do
        echos "${NUMLIST[$coh]}"
        sleep 0.1
    done
	echos "$YE请输入对应的数字：$RE\c"
	read ch
    case $ch in
    1)
    rshy hidemyapplist && ends ;;
    2)
    rshy recoverapplist && ends ;;
    3) 
    echos "$GR正在退出脚本……$RE"; exit 0 ;;        
    *) 
    echos "$GR输入错误，请重新输入$RE"; chlist ;;
    esac
}

suspath() {
echos "$GR你正在使用 一键配置sus路径 功能"
echos "感谢酷安@幽影WF的一键配置sus路径脚本"
echos "正在下载必要文件……$RE"
MODULE_DE="$YSHELL_PATH/Script.sh"
downloader "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/Script.sh"
chmod 755 $MODULE_DE
(sh $MODULE_DE)
MODULE_DE="$YSHELL_PATH/installmodule.zip"
}

switchroot() {
ddQualcomm() {
platform=$(getprop ro.board.platform 2>/dev/null)
if echo "$platform" | grep -qiE '^(msm|apq|qsd)'; then
    echos "$GR检测到高通处理器（平台：$platform）$RE"
    return
fi
hardware=$(getprop ro.hardware 2>/dev/null)
[[ -z "$hardware" ]] && hardware=$(getprop ro.boot.hardware 2>/dev/null)
if echo "$hardware" | grep -qi 'qcom'; then
    echos "$GR检测到高通处理器（硬件：$hardware）$RE" 
    return   
fi
if grep -qiE 'qualcomm|qcom' /proc/cpuinfo 2>/dev/null; then
    echos "$GR检测到高通处理器（来自/proc/cpuinfo）$RE"  
    return  
fi
if [[ -f /system/build.prop ]]; then
    if grep -qiE 'qualcomm|qcom' /system/build.prop 2>/dev/null; then
        echos "$GR检测到高通处理器（来自/system/build.prop）$RE"  
        return      
    fi
fi

echos "$YE未检测到高通处理器$RE"
exit 0
}
ddQualcomm 
echos "$GR你正在使用 快捷切换Root方案 功能"
echos "此功能的实现都在模块内完成"
echos "感谢酷安@Aurora星空_Z的AMMF框架！"
echos "正在下载必要文件……$RE"
downloader "https://github.com/yu13140/RootSwitcher/releases/download/v2.3.0_development/RootSwitcher_v2.3.0_development.zip" "04d23e833db2aa2dde7c37234491230baf6812cc0a6e33cd04799a61806fbd6e"
installer
}

awarmlist() {
clear
detect_magisk
WARMLIST=(
           "$GR你正在使用 一键刷入所需模块 功能$RE"           
           " "
           "$YE非常感谢酷安@Aurora星空_Z的自动化安装模块！$RE"
           " "
           "$GR请手动选择是否把当前所有模块全部删除"
           "1.删除当前所有模块"
           "2.保留当前所有模块(可能会导致隐藏效果不好)$RE"
	       )
	for don in "${!WARMLIST[@]}"; do
        echos "${WARMLIST[$don]}"
        sleep 0.1
    done
    echos "$YE请输入对应的数字：$RE\c"

	read deleteyn
    case $deleteyn in
        1) 
        echos "$GR你选择了删除所有模块$RE"; touch $YSHELL_PATH/delete_all_module ; yuhide ;;
        2) 
        echos "$GR你选择了保留当前模块$RE"; yuhide ;;
        *) 
        echos "$GR输入错误，请重新输入$RE"; awarmlist ;;
    esac
}

ddpeekaboo() {
wfpeekaboo() {
    echos "$YE好像没刷入peekaboo模块呢！请私信酷安@yu13140反馈错误$RE"
    rm -rf $YSHELL_PATH/*
    exit 1
}
ddnohello() {
if [[ $APATCH_VERSION -le 11010 ]] && [[ $APATCH_VERSION -ge 10983 ]]; then
    echos "$YE您的APatch版本太低，不建议您嵌入Nohello模块$RE"
    return 1
fi
rshy --download "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/peekaboo/nohello.kpm" "nohello.kpm"
[[ ! -f $YSHELL_PATH/nohello.kpm ]] && echos "${YE}Nohello未下载成功，取消安装操作$RE" && return 1
insnh=1
}
if [[ ! $ENVIRONMENT = "APatch" ]]; then
    echos "$YE非APatch用户请不要安装peekaboo"
    sleep 2
    return 1
fi

echos "$RED这是一个高危选项，请确保手机自备救砖能力$RE"
echos " "
echos "$GR若你已经确认过风险，并选择安装，请输入 1 "
echos "1.安装                         2.不安装"
echos "请输入对应的选项：$RE\c"

read warnpeekaboo
case $warnpeekaboo in
    1) 
    echos "$GR正在进入安装模块环节$RE" ;;
    *) 
    echos "$YE你选择退出安装peekaboo$RE"; return ;;
esac

echos " "
echos "$YE请输入当前APatch的超级密钥，这不会侵犯您的任何隐私"  
echos "输入超级密钥：$RE\c"
read super_key

if [[ -z $super_key ]]; then
    echos " "
    echos "$YE超级密钥不能为空！"
    echos "请再次输入超级密钥：$RE\c"
    read super_key
fi

if [[ -d /data/data/me.garfieldhan.apatch.next ]] && [[ $NEXT_AP = 1 ]]; then
    APP_AP="me.garfieldhan.apatch.next"
    APP_AP_PATHS=$(pm path $APP_AP | sed -n 's/^package://p' | awk -F'base' '{print $1}')    
elif [[ -d /data/data/me.bmax.apatch ]]  && [[ $NEXT_AP = 0 ]]; then
    APP_AP="me.bmax.apatch"
    APP_AP_PATHS=$(pm path $APP_AP | sed -n 's/^package://p' | awk -F'base' '{print $1}')    
else
    echos "$YE未找到你的APatch，请确认是否安装了APatch管理器$RE"
    return 1
fi
APP_AP_PATH="$APP_AP_PATHS/lib/arm64" 

if [[ ! -d "/dev/block/by-name" ]]; then   
    SITE="/dev/block/bootdevice/by-name"
    if [[ ! -d "/dev/block/bootdevice/by-name" ]]; then
        echos "$YE未检测到分区路径，接下来不会嵌入peekboo$RE"
        return 1
    fi 
else
    SITE="/dev/block/by-name" 
fi
    if [[ $($APP_AP_PATH/libkpatch.so "$super_key" kpm num) -ne 0 ]]; then
        echos "$YE检测到已经装有内核模块"
        echos "如果仍要使用会把已经安装的内核模块删除"
        echos " "
        echos "1.仍要安装                        2.不安装"
        echos "请输入对应的选项：$RE\c"
        read peekabooinit
        case $peekabooinit in
        1) 
        echos "$GR你选择了仍要安装$RE";;
        2) 
        echos "$YE你选择退出安装peekaboo$RE"; return ;;
        *)
        echos "$YE输入错误，默认不安装$RE"; return ;;
        esac
    fi

local version="$APATCH_VERSION"

case 1 in
    $(( version == 11021 )) )
        echos "$WH检测到您正在使用APatch Next(11021)"
        echos "推荐使用cherish_peekaboo_1.5.5"
        echos "正在下载中……$RE"        
        rshy --download "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/peekaboo/cherish_peekaboo_1.5.5.kpm" "peekaboo.kpm"
    ;;
    $(( version >= 10983 && version <= 11010 )) )
       echos "$WH检测到您正在使用APatch($APATCH_VERSION)"
       echos "推荐使用cherish_peekaboo_1.5"
       echos "正在下载中……$RE"
       rshy --download "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/peekaboo/cherish_peekaboo_1.5.kpm" "peekaboo.kpm"
    ;;
    *) 
        echos "$WH检测到您正在使用APatch($APATCH_VERSION)"
        echos "推荐使用cherish_peekaboo_1.5.5"
        echos "正在下载中……$RE"
        rshy --download "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/peekaboo/cherish_peekaboo_1.5.5.kpm" "peekaboo.kpm"
esac

[[ ! -f $YSHELL_PATH/peekaboo.kpm ]] && echos "${YE}peekaboo未下载成功，取消安装操作$RE" && return 1

qchoice "NoHello" "ddnohello"

BOOTAB="$(getprop ro.build.ab_update)"
Partition_location=$(getprop ro.boot.slot_suffix)
if [[ $BOOTAB = "true" ]]; then
    echos "检测到设备支持A/B分区"    
    if [[ "$Partition_location" == "_a" ]]; then
        echos "$GR你目前处于 A 分区$RE"
        position=$(ls -l $SITE/boot_a | awk '{print $NF}')
    elif [[ "$Partition_location" == "_b" ]]; then
        echos "$GR你目前处于 B 分区$RE"
        position=$(ls -l $SITE/boot_b | awk '{print $NF}')        
    fi
else
    position=$(ls -l $SITE/boot | awk '{print $NF}')
fi

cd $YSHELL_PATH
cp -af $APP_AP_PATH/libkptools.so ./kptools
cp -af $APP_AP_PATHS/base.apk ./base.zip
cp -af $APP_AP_PATH/libmagiskboot.so ./magiskboot
command -v ./magiskboot >/dev/null 2>&1 || { >&2 echo "- 未找到APatch目录下的magiskboot文件"; return 1; }
command -v ./kptools >/dev/null 2>&1 || { >&2 echo "- 未找到APatch目录下的Kptools文件"; return 1; }
mkdir ./apatch
unzip base.zip -d ./apatch
chmod -R 777 .
if [[ ! -f /data/data/$APP_AP/new-boot.img ]]; then
    echos "$WH未找到APatch修补后的boot.img！"
    echos "正在提取设备中的boot分区$RE"
    dd if="$position" of="new-boot.img" bs=4M
else
    cp -af /data/data/$APP_AP/new-boot.img ./
fi

./magiskboot unpack new-boot.img
if [ ! $(./kptools -i kernel -l | grep patched=false) ]; then
    ./kptools -u --image kernel --out rekernel
else
    mv kernel rekernel
fi

rm -f kernel
if [[ $insnh = 1 ]]; then
    ./kptools -p -i rekernel -s "$super_key" -k ./apatch/assets/kpimg -o $YSHELL_PATH/kernel -M peekaboo.kpm -V pre-kernel-init -T kpm -M nohello.kpm -V pre-kernel-init -T kpm 2>&1 | tee log.txt
else
    ./kptools -p -i rekernel -s "$super_key" -k ./apatch/assets/kpimg -o $YSHELL_PATH/kernel -M peekaboo.kpm -V pre-kernel-init -T kpm 2>&1 | tee log.txt
fi

if [[ ! $(cat log.txt | grep "patch done") ]]; then
    wfpeekaboo
fi

rm -f rekernel
mv new-boot.img boot.img
./magiskboot repack boot.img
rm -f boot.img
rm -f peekaboo.kpm
mv new-boot.img boot.img

dd if=boot.img of="$position" bs=4M
if [[ $? -ne 0 ]]; then
    echos "$YE未刷入peekaboo模块！$RE"
else
    echos "$GR已成功刷入peekaboo模块$RE"
fi
rm -rf $YSHELL_PATH/*
}

yuhide() {    
    echos "$YE正在下载必要文件中$RE"
    if [[ $Kistune == 1 ]]; then
        murl="https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/Kistune.zip"
    elif [[ $ENVIRONMENT == "APatch" ]]; then
        murl="https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/APatch.zip"
    else
        murl="https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/ARMIAS.zip"
    fi
    rshy --download $murl $MODULE_DE
    rshy nativetest boothash
    installer
    
    if [[ $ENVIRONMENT = "APatch" ]]; then
        qchoice "$YE选择是否需要刷入cherish_peekaboo模块$RE$RED(高危选项)$RE$YE" \
            "ddpeekaboo" "peekaboo" true
    fi

    if [[ $ENVIRONMENT != "APatch" ]]; then   
        qchoice "自动神仙救砖" "select_modules \"20\""
    fi

    qchoice "$YE选择是否安装检测软件10件套" \
        "installapks" "检测软件10件套" true
    
    clear
    echos "$YE正在清理残余垃圾，请稍等……$RE"
    rm -f "$MODULE_DE"
    sleep 1    
    ends
}

menuss() {
case $1 in
    1) 
    clear; menu ;;
    2) 
    clear; gmodules ;;
    3)
    clear; chlist ;;
    4)
    clear; awarmlist ;;
    5)
    clear; installapks; ends ;;
    6)
    clear; extramodule ;;
    7)
    clear; suspath ;;    
    8)
    clear; switchroot ;;    
    f) 
    echos "$GR正在退出脚本$RE"; exit 0 ;;
    *) 
    echos "$YE输入错误，请重新选择$RE"; echos "$GR请等待2秒$RE"; sleep 1.4; selmenu ;;
    esac
}

start() {
	echos "                                        "
	echos "$WH现在时间是：$(date +"%Y年%m月%d日周%w %H时%M分%S秒")$RE"
	echos "$GR您正在使用适用于REDMI 8的简易隐藏root脚本"
    echos "作者：酷安@yu13140$RE"
	echos "                                        "
	echos "$YE本脚本纯免费，如遇到需要缴费使用此脚本的情况"
    echos "说明你被骗了，请立即联系骗子来避免不必要的损失"
    echos "需要给作者打赏，"
    echos "请联系QQ：3031633309或者酷安私信@yu13140"
	echos "                                        "
	echos "注意！！！！！！！！！！！！！！！！"
    echos "在使用脚本前，请你先检查"
    echos "1.手机是否自备救砖能力"
    echos "2.脚本是否使用root权限运行"
    echos "3.国外手机使用此脚本可能没有用(如红魔，索尼)$RE"
    echos "                                        "
	echos "$RED请仔细阅读上面的注意事项$RE"
	echos " "
	echos "${YE}当前版本：${SCRIPT_VERSION}_${PATCH_VERSION}${RE}${WH}"
	echos " "
	echos "- - - - - - -更新日志- - - - - - -"	
	echos "详情请看下载的压缩包内的更新日志"
	sleep 1.2
	echos "                                        $RE"
	echos "$GR若需要使用此脚本，请输入 1"
    echos "若需要退出，请输入 2$RE"
	echos "                                        "
	echos "$YE请输入对应的数字：$RE\c "
	read cf
    case $cf in
    1) 
    echos "$GR正在配置脚本"; echos "这可能需要等待几秒钟……$RE"; sleep 1.4; selmenu ;;
    2) 
    echos "$GR正在退出脚本……$RE"; exit 0 ;;
    *) 
    echos "$GR输入错误，退出脚本$RE"; exit 0 ;;
    esac
}

waitingstart() {
    if [[ "$(whoami)" != "root" ]]; then
        echo "当前脚本的所有者为: $(whoami)"
        echo "- 本脚本未获得 Root 权限，请授权"
        exit 1
    fi

    yudir="$(dirname "$0")"
    STARTSHELL="${YSHELL_PATH}/start.sh"

    if [[ "${yudir}" != "${YSHELL_PATH}" ]]; then
        rm -rf "${YSHELL_PATH}"
        if [[ ! -d "${YSHELL_PATH}" ]]; then
            mkdir -p "${YSHELL_PATH}" || { echo "无法创建目录 ${YSHELL_PATH}"; exit 1; }
            umask 022
        fi

        cp -af "$0" "${STARTSHELL}" || { echo "由于权限问题，脚本复制失败"; exit 1; }
        exec sh "${STARTSHELL}"
    fi
    
    [[ -f "${YSHELL_PATH}/rshy" ]] && rm -f "${YSHELL_PATH}/rshy"
    sed "1,/^# This is the last line of the script/d" "$0" | base64 -d > "${YSHELL_PATH}/rshy"
    chmod 755 "${YSHELL_PATH}/rshy"
    export PATH="${YSHELL_PATH}:$PATH"
}

Initialization() {
rshy --color
sleep 2
clear
}

update() {
local_version=$(rshy --version | tr -d '[:space:]')
remote_version=$(rshy --update | tr -d '[:space:]')

[[ -z $remote_version ]] && echos "$YE当前网速差，不能从云端获取版本号！$RE"

if [[ "$remote_version" != "$local_version" ]]; then
    echos "$WH有新版本：${remote_version} 可以更新！"
    echos "网盘更新地址：https://www.123684.com/s/Wq68jv-Fec4?"
    echos "提取码：rn1C$RE"
    exit 0
fi
}

trap 'rm -rf $YSHELL_PATH; set +m kill -15 $$ 2>&1 >/dev/null' EXIT
waitingstart
rm -f $MODULE_DE
update
detect_environment
clear
Initialization
start
exit 0
# This is the last line of the script
